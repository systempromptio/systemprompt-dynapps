//! AI-activity page routes: everything that reads what the AI actually did.
//!
//! They share one scope contract (`?scope=`/`?range=`), which is why they are
//! one group rather than filed under the entity each happens to list. The
//! composition root merges the table exports and the governance request
//! detail page into the same group. The account history pages read the same
//! conversations under the viewer's own scope.

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use sqlx::PgPool;

use crate::handlers;

pub fn routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/analytics", get(handlers::ssr::analytics_dashboard_page))
        .route("/requests", get(handlers::ssr::analytics_requests_page))
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

pub fn history_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/history", get(handlers::ssr::history_page))
        .route(
            "/history/conversations/{context_id}",
            get(handlers::ssr::history_conversation_page),
        )
}

pub fn history_api_routes() -> Router<Arc<PgPool>> {
    Router::new().route("/api/history/search", get(handlers::ssr::history_search))
}
