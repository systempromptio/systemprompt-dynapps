//! Shared state for the public farm API and the runner service.
//!
//! Built once at boot from the profile, the manifest config and the composed
//! services tree. It holds no executor handle: runs are rows on core's
//! `agent_runs` table, claimed by
//! [`crate::services::runner::FarmRunnerService`].

mod limits;
mod paths;

use std::sync::Arc;

use sqlx::PgPool;

pub use limits::RunnerLimits;
pub use paths::FarmPaths;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayEndpoints {
    pub internal_base: String,
    pub external_base: String,
}

impl GatewayEndpoints {
    #[must_use]
    pub fn govern_url(&self) -> String {
        format!("{}/api/public/hooks/govern", self.internal_base)
    }

    #[must_use]
    pub fn track_url(&self) -> String {
        format!("{}/api/public/hooks/track", self.internal_base)
    }
}

#[derive(Debug)]
pub struct FarmState {
    pub pool: Arc<PgPool>,
    pub paths: FarmPaths,
    pub gateway: GatewayEndpoints,
    pub limits: RunnerLimits,
}

#[must_use]
pub fn build_farm_state(
    pool: Arc<PgPool>,
    paths: FarmPaths,
    gateway: GatewayEndpoints,
    limits: RunnerLimits,
) -> Arc<FarmState> {
    Arc::new(FarmState {
        pool,
        paths,
        gateway,
        limits,
    })
}
