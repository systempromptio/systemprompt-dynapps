//! The runner machine pool.
//!
//! Take a stopped `farm_runner` machine and launch it with one run's env, or
//! create one when none is free; stop it after the run, and destroy it when
//! the pool already holds `pool_size` machines.
//!
//! Machines taken by in-flight runs are held on the shared [`RemoteRuns`], so
//! two runs on the same API machine never launch the same runner. Every unset
//! `fly:` field is read from the Fly runtime env (`FLY_APP_NAME`, `FLY_REGION`,
//! `FLY_IMAGE_REF`, `FLY_MACHINE_ID`), which is why each launch re-sends the
//! image: a pooled machine always runs the image the API runs.

use std::collections::BTreeMap;
use std::sync::Arc;

use super::api::{Machine, MachineLaunch, MachinesClient, api_token, machine_config};
use super::protocol::{API_BASE_ENV, RUN_ID_ENV, WORKER_TOKEN_ENV};
use super::registry::RemoteRuns;
use crate::config::FlyRunnerConfig;
use crate::error::{FarmError, FarmResult};

const STOPPED: &str = "stopped";

/// The Fly facts a launch needs, resolved once per run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlyTarget {
    pub app: String,
    pub region: String,
    pub image: String,
    pub api_base: String,
}

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

fn required(value: Option<String>, what: &str) -> FarmResult<String> {
    value.ok_or_else(|| FarmError::Runner(format!("fly runner: {what} is not set")))
}

#[must_use]
pub fn private_api_base(app: &str, machine_id: &str, port: u16) -> String {
    format!("http://{machine_id}.vm.{app}.internal:{port}")
}

pub fn resolve_target(config: &FlyRunnerConfig) -> FarmResult<FlyTarget> {
    let app = required(config.app.clone().or_else(|| env("FLY_APP_NAME")), "app")?;
    let region = required(
        config.region.clone().or_else(|| env("FLY_REGION")),
        "region",
    )?;
    let image = required(
        config.image.clone().or_else(|| env("FLY_IMAGE_REF")),
        "image",
    )?;
    let api_base = match config.api_base.clone() {
        Some(base) => base,
        None => private_api_base(
            &app,
            &required(env("FLY_MACHINE_ID"), "FLY_MACHINE_ID")?,
            config.api_port,
        ),
    };
    Ok(FlyTarget {
        app,
        region,
        image,
        api_base: api_base.trim_end_matches('/').to_owned(),
    })
}

#[must_use]
pub fn run_env(run_id: &str, api_base: &str, token: &str) -> BTreeMap<String, String> {
    BTreeMap::from([
        (RUN_ID_ENV.to_owned(), run_id.to_owned()),
        (API_BASE_ENV.to_owned(), api_base.to_owned()),
        (WORKER_TOKEN_ENV.to_owned(), token.to_owned()),
    ])
}

#[derive(Debug)]
pub struct RunnerPool {
    client: MachinesClient,
    config: FlyRunnerConfig,
    runs: Arc<RemoteRuns>,
}

impl RunnerPool {
    pub fn new(
        config: &FlyRunnerConfig,
        target: &FlyTarget,
        runs: Arc<RemoteRuns>,
        secrets: &dyn systemprompt::traits::SecretStore,
    ) -> FarmResult<Self> {
        let token = api_token(secrets).ok_or_else(|| {
            FarmError::Runner("fly runner: no fly_api_token secret or FLY_API_TOKEN".to_owned())
        })?;
        Ok(Self {
            client: MachinesClient::new(config, &target.app, &token)?,
            config: config.clone(),
            runs,
        })
    }

    pub async fn acquire(&self, region: &str, launch: &MachineLaunch) -> FarmResult<Machine> {
        let config = machine_config(&self.config, launch);
        for machine in self.client.list_pool().await? {
            if machine.state != STOPPED || !self.runs.take_machine(&machine.id) {
                continue;
            }
            match self.relaunch(&machine.id, config.clone()).await {
                Ok(started) => return self.started(started).await,
                Err(error) => {
                    self.runs.give_back_machine(&machine.id);
                    tracing::warn!(machine = %machine.id, %error, "pooled runner not launched");
                },
            }
        }
        let created = self.client.create(region, config).await?;
        self.runs.take_machine(&created.id);
        self.started(created).await
    }

    // Why: the Fly Machines API leaves a stopped machine stopped after a
    // config update ("leaving the new version stopped"), so the pooled
    // runner is started explicitly once the update has settled.
    // JSON: the machine config is the Fly Machines API wire body.
    async fn relaunch(&self, id: &str, config: serde_json::Value) -> FarmResult<Machine> {
        let updated = self.client.update(id, config).await?;
        self.client.wait(&updated, "stopped").await?;
        self.client.start(id).await?;
        Ok(updated)
    }

    async fn started(&self, machine: Machine) -> FarmResult<Machine> {
        if let Err(error) = self.client.wait(&machine, "started").await {
            self.release(&machine.id).await;
            return Err(error);
        }
        Ok(machine)
    }

    pub async fn release(&self, id: &str) {
        if let Err(error) = self.client.stop(id).await {
            tracing::warn!(machine = %id, %error, "runner machine not stopped");
        }
        let pooled = self.client.list_pool().await.map(|machines| {
            machines
                .iter()
                .filter(|m| m.id != id && m.state == STOPPED)
                .count()
        });
        self.runs.give_back_machine(id);
        let surplus = matches!(pooled, Ok(count) if count >= self.config.pool_size);
        if surplus && let Err(error) = self.client.destroy(id).await {
            tracing::warn!(machine = %id, %error, "surplus runner machine not destroyed");
        }
    }
}
