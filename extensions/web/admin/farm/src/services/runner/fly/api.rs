//! A small client for the Fly Machines REST API: the calls the runner pool
//! makes and nothing else.
//!
//! The token is the `fly_api_token` secret, or `FLY_API_TOKEN` in the
//! environment. A deploy or org token minted by `fly tokens create` already
//! carries its `FlyV1` scheme and is sent as it is; a bare token is sent as
//! a bearer.

use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::config::FlyRunnerConfig;
use crate::error::{FarmError, FarmResult};

pub const TOKEN_SECRET: &str = "fly_api_token";
pub const TOKEN_ENV: &str = "FLY_API_TOKEN";
pub const POOL_METADATA_KEY: &str = "farm_runner";
pub const POOL_METADATA_VALUE: &str = "pool";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(90);
const WAIT_SECONDS: u64 = 60;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Machine {
    pub id: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub region: String,
    #[serde(default)]
    pub instance_id: Option<String>,
}

/// What a runner machine is launched with for one run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MachineLaunch {
    pub image: String,
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct MachinesClient {
    http: reqwest::Client,
    base: String,
    app: String,
    auth: String,
}

#[must_use]
pub fn api_token() -> Option<String> {
    std::env::var(TOKEN_ENV)
        .ok()
        .or_else(|| {
            systemprompt::config::SecretsBootstrap::get()
                .ok()
                .and_then(|s| s.get(TOKEN_SECRET).cloned())
        })
        .filter(|t| !t.trim().is_empty())
}

#[must_use]
pub fn auth_header(token: &str) -> String {
    let token = token.trim();
    if token.starts_with("FlyV1 ") || token.starts_with("Bearer ") {
        token.to_owned()
    } else {
        format!("Bearer {token}")
    }
}

#[must_use]
pub fn machine_config(config: &FlyRunnerConfig, launch: &MachineLaunch) -> Value {
    json!({
        "image": launch.image,
        "env": launch.env,
        "guest": {
            "cpu_kind": config.cpu_kind,
            "cpus": config.cpus,
            "memory_mb": config.memory_mb,
        },
        "init": { "exec": [config.worker_bin] },
        "restart": { "policy": "no" },
        "auto_destroy": false,
        "metadata": { POOL_METADATA_KEY: POOL_METADATA_VALUE },
    })
}

impl MachinesClient {
    pub fn new(config: &FlyRunnerConfig, app: &str, token: &str) -> FarmResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|source| FarmError::RunnerHttp {
                at: "machines client".to_owned(),
                source,
            })?;
        Ok(Self {
            http,
            base: format!(
                "{}/v1/apps/{app}",
                config.machines_api.trim_end_matches('/')
            ),
            app: app.to_owned(),
            auth: auth_header(token),
        })
    }

    #[must_use]
    pub fn app(&self) -> &str {
        &self.app
    }

    async fn send<T: for<'de> Deserialize<'de>>(
        &self,
        builder: reqwest::RequestBuilder,
        what: &str,
    ) -> FarmResult<Option<T>> {
        let response = builder
            .header(reqwest::header::AUTHORIZATION, &self.auth)
            .send()
            .await
            .map_err(|source| FarmError::RunnerHttp {
                at: what.to_owned(),
                source,
            })?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|source| FarmError::RunnerHttp {
                at: what.to_owned(),
                source,
            })?;
        if !status.is_success() {
            return Err(FarmError::Runner(format!("{what}: HTTP {status}: {body}")));
        }
        if body.trim().is_empty() {
            return Ok(None);
        }
        serde_json::from_str(&body)
            .map(Some)
            .map_err(|source| FarmError::Json {
                at: what.to_owned(),
                source,
            })
    }

    pub async fn list_pool(&self) -> FarmResult<Vec<Machine>> {
        let url = format!(
            "{}/machines?metadata.{POOL_METADATA_KEY}={POOL_METADATA_VALUE}",
            self.base
        );
        Ok(self
            .send(self.http.get(url), "list runner machines")
            .await?
            .unwrap_or_default())
    }

    pub async fn create(&self, region: &str, config: Value) -> FarmResult<Machine> {
        let body = json!({ "region": region, "config": config });
        self.send(
            self.http
                .post(format!("{}/machines", self.base))
                .json(&body),
            "create runner machine",
        )
        .await?
        .ok_or_else(|| FarmError::Runner("create runner machine: empty reply".to_owned()))
    }

    pub async fn update(&self, id: &str, config: Value) -> FarmResult<Machine> {
        let body = json!({ "config": config });
        self.send(
            self.http
                .post(format!("{}/machines/{id}", self.base))
                .json(&body),
            "update runner machine",
        )
        .await?
        .ok_or_else(|| FarmError::Runner("update runner machine: empty reply".to_owned()))
    }

    pub async fn wait(&self, machine: &Machine, state: &str) -> FarmResult<()> {
        let mut url = format!(
            "{}/machines/{}/wait?state={state}&timeout={WAIT_SECONDS}",
            self.base, machine.id
        );
        if let Some(instance) = &machine.instance_id {
            url.push_str("&instance_id=");
            url.push_str(instance);
        }
        self.send::<Value>(self.http.get(url), "wait for runner machine")
            .await
            .map(|_| ())
    }

    pub async fn stop(&self, id: &str) -> FarmResult<()> {
        self.send::<Value>(
            self.http.post(format!("{}/machines/{id}/stop", self.base)),
            "stop runner machine",
        )
        .await
        .map(|_| ())
    }

    pub async fn destroy(&self, id: &str) -> FarmResult<()> {
        self.send::<Value>(
            self.http
                .delete(format!("{}/machines/{id}?force=true", self.base)),
            "destroy runner machine",
        )
        .await
        .map(|_| ())
    }
}
