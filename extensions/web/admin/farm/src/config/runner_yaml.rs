//! Where a farm run executes: `runner.mode` in the farms file.
//!
//! `local` (the default) spawns the host as a child of the API process.
//! `fly_machine` runs each claimed run on its own Fly Machine: the API takes
//! a stopped machine from a pool tagged `farm-runner` (or creates one from
//! the app's current image), starts it with the run's id, a run-scoped
//! worker token and the API base it reports back to, and waits for the
//! worker's terminal result. Every `fly:` field has a default read from the
//! Fly runtime environment, so `mode: fly_machine` alone is enough on Fly.
//! `fly.egress_allow` adds hosts, IPs or CIDRs a runner may reach beyond
//! Fly's private network and the run's own URLs (see `runner::fly::egress`).
//!
//! The services tree is shared by every profile, so the mode can also be set
//! per environment with [`RUNNER_MODE_ENV`], which wins over the file.

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::farms_yaml::{FARMS_FILE, RunnerConfig, parse_farms_doc};

pub const RUNNER_MODE_ENV: &str = "FARM_RUNNER_MODE";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunnerMode {
    #[default]
    Local,
    FlyMachine,
}

impl RunnerMode {
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "local" => Some(Self::Local),
            "fly_machine" => Some(Self::FlyMachine),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::FlyMachine => "fly_machine",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlyRunnerConfig {
    #[serde(default)]
    pub app: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default = "default_machines_api")]
    pub machines_api: String,
    #[serde(default)]
    pub api_base: Option<String>,
    #[serde(default = "default_api_port")]
    pub api_port: u16,
    #[serde(default = "default_pool_size")]
    pub pool_size: usize,
    #[serde(default = "default_cpu_kind")]
    pub cpu_kind: String,
    #[serde(default = "default_cpus")]
    pub cpus: u32,
    #[serde(default = "default_memory_mb")]
    pub memory_mb: u32,
    #[serde(default = "default_worker_bin")]
    pub worker_bin: String,
    #[serde(default = "default_heartbeat_timeout")]
    pub heartbeat_timeout_seconds: u64,
    #[serde(default = "default_start_timeout")]
    pub start_timeout_seconds: u64,
    #[serde(default)]
    pub egress_allow: Vec<String>,
}

impl Default for FlyRunnerConfig {
    fn default() -> Self {
        Self {
            app: None,
            region: None,
            image: None,
            machines_api: default_machines_api(),
            api_base: None,
            api_port: default_api_port(),
            pool_size: default_pool_size(),
            cpu_kind: default_cpu_kind(),
            cpus: default_cpus(),
            memory_mb: default_memory_mb(),
            worker_bin: default_worker_bin(),
            heartbeat_timeout_seconds: default_heartbeat_timeout(),
            start_timeout_seconds: default_start_timeout(),
            egress_allow: Vec::new(),
        }
    }
}

fn default_machines_api() -> String {
    "https://api.machines.dev".to_owned()
}

const fn default_api_port() -> u16 {
    systemprompt::models::defaults::DEFAULT_API_PORT
}

const fn default_pool_size() -> usize {
    2
}

fn default_cpu_kind() -> String {
    "shared".to_owned()
}

const fn default_cpus() -> u32 {
    2
}

const fn default_memory_mb() -> u32 {
    2048
}

fn default_worker_bin() -> String {
    "/app/bin/systemprompt-farm-worker".to_owned()
}

const fn default_heartbeat_timeout() -> u64 {
    90
}

const fn default_start_timeout() -> u64 {
    180
}

#[must_use]
pub fn read_runner_config(services_root: &Path) -> RunnerConfig {
    let from_file = std::fs::read_to_string(services_root.join(FARMS_FILE))
        .ok()
        .and_then(|text| match parse_farms_doc(&text) {
            Ok(doc) => doc.runner,
            Err(error) => {
                tracing::error!(%error, "farms file unreadable; the runner uses its defaults");
                None
            },
        })
        .unwrap_or_default();
    with_mode_override(from_file, std::env::var(RUNNER_MODE_ENV).ok().as_deref())
}

#[must_use]
pub fn with_mode_override(mut config: RunnerConfig, value: Option<&str>) -> RunnerConfig {
    let Some(raw) = value.filter(|v| !v.trim().is_empty()) else {
        return config;
    };
    if let Some(mode) = RunnerMode::parse(raw) {
        config.mode = mode;
    } else {
        tracing::error!(
            variable = RUNNER_MODE_ENV,
            value = raw,
            "runner mode override is not local|fly_machine"
        );
    }
    config
}
