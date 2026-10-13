//! Shared state for the public farm API and the runner service.
//!
//! Built once at boot from the profile, the manifest config and the composed
//! services tree. It holds no executor handle: runs are rows on core's
//! `agent_runs` table, claimed by
//! [`crate::services::runner::FarmRunnerService`].

mod limits;
mod paths;

use std::path::PathBuf;
use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::config::AppPaths;

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

#[must_use]
pub fn build_farm_state_from_app(
    pool: Arc<PgPool>,
    app_paths: Option<&AppPaths>,
) -> Arc<FarmState> {
    build_farm_state(
        pool,
        farm_paths(app_paths),
        gateway_endpoints(),
        RunnerLimits::default(),
    )
}

fn farm_paths(app_paths: Option<&AppPaths>) -> FarmPaths {
    let data_root = app_paths.map_or_else(
        || std::env::temp_dir().join("systemprompt-agent-farm"),
        |paths| paths.storage().data().join("agent-farm"),
    );
    let composed = systemprompt::loader::ServicesRootBootstrap::get().map_or_else(
        || {
            app_paths.map_or_else(
                || PathBuf::from("services"),
                |paths| paths.system().services().to_path_buf(),
            )
        },
        |root| root.path.clone(),
    );
    FarmPaths::new(data_root, composed)
}

fn gateway_endpoints() -> GatewayEndpoints {
    systemprompt::manifest::Config::get().map_or_else(
        |e| {
            tracing::error!(error = %e, "Config unavailable for the agent farm gateway base");
            GatewayEndpoints {
                internal_base: String::new(),
                external_base: String::new(),
            }
        },
        |config| GatewayEndpoints {
            internal_base: config.api_internal_url.trim_end_matches('/').to_owned(),
            external_base: config.api_external_url.trim_end_matches('/').to_owned(),
        },
    )
}
