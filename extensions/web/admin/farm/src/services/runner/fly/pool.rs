//! The runner machine pool.
//!
//! Take a stopped `farm_runner` machine and launch it with one run's env, or
//! create one when none is free; stop it after the run, and destroy it when
//! the pool already holds `pool_size` machines.
//!
//! Machines taken by this process are held in an in-process set, so two runs
//! on the same API machine never launch the same runner. Every unset `fly:`
//! field is read from the Fly runtime env (`FLY_APP_NAME`, `FLY_REGION`,
//! `FLY_IMAGE_REF`, `FLY_MACHINE_ID`), which is why each launch re-sends the
//! image: a pooled machine always runs the image the API runs.

use std::collections::{BTreeMap, HashSet};
use std::sync::{LazyLock, Mutex};

use super::api::{Machine, MachineLaunch, MachinesClient, api_token, machine_config};
use super::protocol::{API_BASE_ENV, RUN_ID_ENV, WORKER_TOKEN_ENV};
use crate::config::FlyRunnerConfig;
use crate::error::{FarmError, FarmResult};

static TAKEN: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

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

fn take(id: &str) -> bool {
    TAKEN.lock().is_ok_and(|mut set| set.insert(id.to_owned()))
}

fn give_back(id: &str) {
    if let Ok(mut set) = TAKEN.lock() {
        set.remove(id);
    }
}

#[derive(Debug)]
pub struct RunnerPool {
    client: MachinesClient,
    config: FlyRunnerConfig,
}

impl RunnerPool {
    pub fn new(config: &FlyRunnerConfig, target: &FlyTarget) -> FarmResult<Self> {
        let token = api_token().ok_or_else(|| {
            FarmError::Runner("fly runner: no fly_api_token secret or FLY_API_TOKEN".to_owned())
        })?;
        Ok(Self {
            client: MachinesClient::new(config, &target.app, &token)?,
            config: config.clone(),
        })
    }

    pub async fn acquire(&self, region: &str, launch: &MachineLaunch) -> FarmResult<Machine> {
        let config = machine_config(&self.config, launch);
        for machine in self.client.list_pool().await? {
            if machine.state != STOPPED || !take(&machine.id) {
                continue;
            }
            match self.client.update(&machine.id, config.clone()).await {
                Ok(started) => return self.started(started).await,
                Err(error) => {
                    give_back(&machine.id);
                    tracing::warn!(machine = %machine.id, %error, "pooled runner not launched");
                },
            }
        }
        let created = self.client.create(region, config).await?;
        take(&created.id);
        self.started(created).await
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
        give_back(id);
        let surplus = matches!(pooled, Ok(count) if count >= self.config.pool_size);
        if surplus && let Err(error) = self.client.destroy(id).await {
            tracing::warn!(machine = %id, %error, "surplus runner machine not destroyed");
        }
    }
}
