//! Shared state for the public farm API and the runner service.
//!
//! Built once at boot from the profile, the manifest config and the composed
//! services tree. It holds no executor handle: runs are rows on core's
//! `agent_runs` table, claimed by
//! [`crate::services::runner::FarmRunnerService`].

mod judge;
mod limits;
mod paths;

use std::path::PathBuf;
use std::sync::Arc;

use crate::services::runner::fly::registry::RemoteRuns;

use sqlx::PgPool;
use systemprompt::ai::AiService;
use systemprompt::config::AppPaths;
use systemprompt::identifiers::InstanceId;
use systemprompt::storage::{FileStorageBackend, build_file_storage};
use systemprompt::traits::FileStorage;

pub use judge::{JudgeSource, read_evals_config};
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

pub struct FarmState {
    pub pool: Arc<PgPool>,
    pub paths: FarmPaths,
    pub gateway: GatewayEndpoints,
    pub limits: RunnerLimits,
    pub judge: JudgeSource,
    pub artefacts: Arc<dyn FileStorage>,
    pub instance: InstanceId,
    pub runs: Arc<RemoteRuns>,
}

impl std::fmt::Debug for FarmState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FarmState")
            .field("paths", &self.paths)
            .field("gateway", &self.gateway)
            .field("limits", &self.limits)
            .field("instance", &self.instance)
            .finish_non_exhaustive()
    }
}

/// What the composition root hands a [`FarmState`] from the runtime.
///
/// Core's file storage (eval files), this replica's identity (recorded on
/// the eval runs it drives) and the in-flight remote run registry every
/// `FarmState` in the process shares.
#[derive(Clone)]
pub struct FarmHost {
    pub artefacts: Arc<dyn FileStorage>,
    pub instance: InstanceId,
    pub runs: Arc<RemoteRuns>,
}

impl std::fmt::Debug for FarmHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FarmHost")
            .field("instance", &self.instance)
            .finish_non_exhaustive()
    }
}

#[must_use]
pub fn build_farm_state(
    pool: Arc<PgPool>,
    paths: FarmPaths,
    gateway: GatewayEndpoints,
    limits: RunnerLimits,
) -> Arc<FarmState> {
    let artefacts = build_file_storage(FileStorageBackend::Local {
        root: paths.data_root.clone(),
    });
    Arc::new(FarmState {
        pool,
        paths,
        gateway,
        limits,
        judge: JudgeSource::default(),
        artefacts,
        instance: InstanceId::new("local"),
        runs: Arc::new(RemoteRuns::default()),
    })
}

#[must_use]
pub fn build_farm_state_from_app(
    pool: Arc<PgPool>,
    app_paths: Option<&AppPaths>,
    host: FarmHost,
) -> Arc<FarmState> {
    build_farm_state_with_ai(pool, app_paths, None, host)
}

#[must_use]
pub fn build_farm_state_with_ai(
    pool: Arc<PgPool>,
    app_paths: Option<&AppPaths>,
    ai: Option<Arc<AiService>>,
    host: FarmHost,
) -> Arc<FarmState> {
    let paths = farm_paths(app_paths);
    let judge = JudgeSource {
        config: read_evals_config(&paths.composed_services_root),
        ai,
    };
    Arc::new(FarmState {
        pool,
        paths,
        gateway: gateway_endpoints(),
        limits: RunnerLimits::default(),
        judge,
        artefacts: host.artefacts,
        instance: host.instance,
        runs: host.runs,
    })
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
