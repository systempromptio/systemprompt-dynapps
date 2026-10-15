//! The agent farm's public run API, nested at `/api/public/agent-farm`, the
//! farm state it runs over, and the one place that state is started at boot.
//!
//! The router accepts a session cookie or a personal access token on every
//! route: a remote client launching runs holds a PAT, the dashboard's own
//! scripts a session. Either resolves to the same [`UserContext`] the admin
//! plane builds, and the handlers authorise from it.
//!
//! [`UserContext`]: systemprompt_web_admin_core::types::UserContext

use std::sync::Arc;

use axum::Router;
use axum::extract::{Request, State};
use axum::middleware::{self as axum_middleware, Next};
use axum::response::Response;
use systemprompt::config::AppPaths;
use systemprompt::database::DbPool;
use systemprompt::prelude::PgPool;
use systemprompt::traits::{JobContext, RunnerService};
use systemprompt_web_admin_core::ssr::viewer_tz::with_viewer_tz;
use systemprompt_web_admin_farm::routes::public::agent_farm_public_routes;
use systemprompt_web_admin_farm::routes::worker::agent_farm_worker_routes;
use systemprompt_web_admin_farm::services::farm_sync::boot::sync_missing_checkouts;
use systemprompt_web_admin_farm::services::farm_sync::seed_farms_from_yaml;
use systemprompt_web_admin_farm::services::runner::FarmRunnerService;
use systemprompt_web_admin_farm::services::runner::report::backfill_missing_reports;
use systemprompt_web_admin_farm::state::{FarmState, build_farm_state_from_app};
use tokio_util::sync::CancellationToken;

use super::admin;

pub(crate) const FARM_PUBLIC_PREFIX: &str = "/agent-farm";
pub(crate) const FARM_WORKER_PREFIX: &str = "/agent-farm-worker";

pub fn farm_public_router(read_pool: Arc<PgPool>, state: Arc<FarmState>) -> Router {
    agent_farm_public_routes()
        .with_state(state)
        .layer(axum_middleware::from_fn(
            systemprompt_web_admin_core::middleware::gates::require_auth_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            read_pool,
            farm_context_middleware,
        ))
}

pub(crate) fn farm_worker_router() -> Router {
    agent_farm_worker_routes()
}

async fn farm_context_middleware(
    State(pool): State<Arc<PgPool>>,
    mut request: Request,
    next: Next,
) -> Result<Response, admin::PatRejection> {
    let inputs = admin::SessionInputs::from_request(&request, true);
    let Some(session) = admin::resolve_session(&pool, inputs).await? else {
        return Ok(next.run(request).await);
    };
    let (ctx, tz) = admin::resolve_user_context(&pool, session).await;
    request.extensions_mut().insert(ctx);
    Ok(with_viewer_tz(tz, next.run(request)).await)
}

pub(crate) fn start(state: &Arc<FarmState>, app_paths: Option<&AppPaths>) {
    if tokio::runtime::Handle::try_current().is_err() {
        tracing::error!("No async runtime at router build; the agent farms are not seeded");
        return;
    }
    let services = app_paths.map_or_else(
        || state.paths.composed_services_root.clone(),
        |paths| paths.system().services().to_path_buf(),
    );
    let state = Arc::clone(state);
    tokio::spawn(async move {
        match seed_farms_from_yaml(&state.pool, &services).await {
            Ok(count) => tracing::info!(farms = count, "agent farms seeded"),
            Err(e) => tracing::error!(error = %e, "agent farm seed failed"),
        }
        let restored = sync_missing_checkouts(&state).await;
        tracing::info!(farms = restored, "agent farm checkouts restored");
        let reported = backfill_missing_reports(&state.pool).await;
        tracing::info!(runs = reported, "missing farm run reports backfilled");
    });
}

pub(crate) struct FarmRunner;

#[async_trait::async_trait]
impl RunnerService for FarmRunner {
    fn name(&self) -> &'static str {
        "agent_farm_runner"
    }

    async fn run(&self, ctx: JobContext, shutdown: CancellationToken) {
        let pool = match ctx.get::<DbPool>() {
            Ok(db) => db.write_pool(),
            Err(e) => {
                tracing::error!(error = %e, "agent farm runner has no database");
                return;
            },
        };
        let paths = ctx.get::<Arc<AppPaths>>().ok().map(Arc::clone);
        let state = build_state(pool, paths.as_deref());
        FarmRunnerService::new(state).run(ctx, shutdown).await;
    }
}

pub fn build_state(pool: Arc<PgPool>, app_paths: Option<&AppPaths>) -> Arc<FarmState> {
    build_farm_state_from_app(pool, app_paths)
}
