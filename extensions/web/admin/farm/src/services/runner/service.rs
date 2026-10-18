//! [`FarmRunnerService`]: the farm's [`RunnerService`], started by the web
//! extension's `runner_services()` on every node whose role runs runners.
//!
//! It claims `agent_runs` rows of origin [`ORIGIN_KIND`] (every farm run's
//! `origin_kind`) through core's
//! [`ClaimLoop`], at most `RunnerLimits::max_concurrent` at once, and hands
//! each to the farm's handler. Beside the loop it reconciles farm rows whose
//! `agent_runs` row turned terminal without a handler. The handler's runner
//! mode (local child or Fly Machine) is read from the farms file once, here.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use systemprompt::agent_runs::AgentRunRepository;
use systemprompt::agent_runs::credentials::RunCredentialService;
use systemprompt::agent_runs::governance::HookBases;
use systemprompt::agent_runs::repository::RunCredentialRepositories;
use systemprompt::agent_runs::service::{ClaimLoop, RunHandler, claimer_id};
use systemprompt::database::DbPool;
use systemprompt::traits::{JobContext, RunnerService};
use systemprompt::users::ApiKeyLifetime;
use tokio_util::sync::CancellationToken;

use super::handler::FarmRunHandler;
use super::observer;
use crate::config::{RunnerMode, read_runner_config};
use crate::state::FarmState;

pub const ORIGIN_KIND: &str = "agent_farm_run";
const SERVICE_NAME: &str = "agent-farm";
const RECONCILE_EVERY: Duration = Duration::from_secs(30);
const HOSTNAME_ENV: &str = "HOSTNAME";
const FALLBACK_HOSTNAME: &str = "localhost";

#[derive(Debug, Clone)]
pub struct FarmRunnerService {
    state: Arc<FarmState>,
}

impl FarmRunnerService {
    #[must_use]
    pub const fn new(state: Arc<FarmState>) -> Self {
        Self { state }
    }

    fn handler(&self, db: &DbPool, issuer: RunIssuer) -> Arc<dyn RunHandler> {
        let gateway = &self.state.gateway;
        let runner = read_runner_config(&self.state.paths.composed_services_root);
        let fly = (runner.mode == RunnerMode::FlyMachine).then(|| runner.fly.unwrap_or_default());
        tracing::info!(mode = runner.mode.as_str(), "farm runner mode");
        Arc::new(FarmRunHandler {
            state: Arc::clone(&self.state),
            credentials: RunCredentialService::new(
                RunCredentialRepositories::new(db),
                issuer.jwt_issuer,
                issuer.key_lifetime,
            ),
            hooks: HookBases {
                govern_url: gateway.govern_url(),
                track_url: gateway.track_url(),
            },
            fly,
        })
    }
}

fn hostname() -> String {
    std::env::var(HOSTNAME_ENV)
        .ok()
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| FALLBACK_HOSTNAME.to_owned())
}

struct RunIssuer {
    jwt_issuer: String,
    key_lifetime: ApiKeyLifetime,
}

fn run_issuer() -> Option<RunIssuer> {
    match systemprompt::manifest::Config::get() {
        Ok(config) => Some(RunIssuer {
            jwt_issuer: config.jwt_issuer.clone(),
            key_lifetime: ApiKeyLifetime::from_max_days(config.api_key_max_lifetime_days),
        }),
        Err(error) => {
            tracing::error!(%error, "config unavailable; no farm run will be claimed");
            None
        },
    }
}

#[async_trait]
impl RunnerService for FarmRunnerService {
    fn name(&self) -> &'static str {
        SERVICE_NAME
    }

    async fn run(&self, ctx: JobContext, shutdown: CancellationToken) {
        let Ok(db) = ctx.get::<DbPool>().cloned() else {
            tracing::error!("farm runner has no DbPool; no farm run will be claimed");
            return;
        };
        let Some(issuer) = run_issuer() else {
            return;
        };
        let mut claim = ClaimLoop::new(
            AgentRunRepository::from_pool(Arc::clone(&self.state.pool)),
            ORIGIN_KIND,
            claimer_id(&hostname()),
        );
        claim.max_concurrent = self.state.limits.max_concurrent.max(1);
        let sweep = reconcile_loop(Arc::clone(&self.state), shutdown.clone());
        tokio::join!(claim.run(shutdown, self.handler(&db, issuer)), sweep);
    }
}

async fn reconcile_loop(state: Arc<FarmState>, shutdown: CancellationToken) {
    loop {
        match observer::reconcile(&state).await {
            Ok(0) => {},
            Ok(count) => tracing::info!(count, "farm runs reconciled with agent_runs"),
            Err(error) => tracing::error!(%error, "farm run reconcile failed"),
        }
        tokio::select! {
            () = shutdown.cancelled() => return,
            () = tokio::time::sleep(RECONCILE_EVERY) => {},
        }
    }
}
