//! AI-activity page routes: everything that reads what the AI actually did.
//! They share one scope contract (`?scope=`/`?range=`), which is why they are
//! one group rather than filed under the entity each happens to list.

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use sqlx::PgPool;

use crate::handlers;

pub(super) fn ai_activity_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/analytics", get(handlers::ssr::analytics_dashboard_page))
        .merge(crate::routes::ssr_export::routes())
        .route("/requests", get(handlers::ssr::analytics_requests_page))
        .merge(systemprompt_web_admin_governance::routes::ssr_governance::request_detail_routes())
        .route("/sessions", get(handlers::ssr::sessions_list_page))
        .route(
            "/sessions/{session_id}",
            get(handlers::ssr::session_detail_page),
        )
        .route("/traces", get(handlers::ssr::perf_traces_page))
        .route(
            "/traces/{trace_id}",
            get(handlers::ssr::perf_trace_detail_page),
        )
        .route("/tools", get(handlers::ssr::tools_page))
        .route("/artifacts", get(handlers::ssr::artifacts_page))
        .route(
            "/artifacts/{artifact_id}",
            get(handlers::ssr::artifact_detail_page),
        )
        .route(
            "/artifacts/{artifact_id}/preview",
            get(handlers::ssr::artifact_preview),
        )
        .route("/conversations", get(handlers::ssr::conversations_page))
        .route("/contexts", get(handlers::ssr::skills_contexts_page))
        .route(
            "/contexts/{context_id}",
            get(handlers::ssr::context_detail_page),
        )
}
