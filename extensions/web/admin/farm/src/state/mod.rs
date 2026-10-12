//! Shared state for the public farm API and the runner, built once at boot
//! from the profile, the manifest config and the composed services tree.

mod limits;
mod paths;

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use sqlx::PgPool;

use crate::services::runner::RunnerHandle;

pub use limits::RunnerLimits;
pub use paths::FarmPaths;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayEndpoints {
    pub internal_base: String,
    pub external_base: String,
}

#[derive(Debug)]
pub struct FarmState {
    pub pool: Arc<PgPool>,
    pub paths: FarmPaths,
    pub gateway: GatewayEndpoints,
    pub limits: RunnerLimits,
    pub claude_bin: PathBuf,
    pub runner: OnceLock<Arc<RunnerHandle>>,
}

impl FarmState {
    #[must_use]
    pub fn runner(&self) -> Option<&Arc<RunnerHandle>> {
        self.runner.get()
    }
}

#[must_use]
pub fn build_farm_state(
    pool: Arc<PgPool>,
    paths: FarmPaths,
    gateway: GatewayEndpoints,
    limits: RunnerLimits,
    claude_bin: PathBuf,
) -> Arc<FarmState> {
    Arc::new(FarmState {
        pool,
        paths,
        gateway,
        limits,
        claude_bin,
        runner: OnceLock::new(),
    })
}
