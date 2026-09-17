//! Analysis routes: skills, conversations, marketplace versions, evaluation
//! and the publication decisions taken from a marketplace's Distribution view.

use crate::handlers;
use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;
use std::sync::Arc;

pub(super) fn routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/analysis/inventory/sync",
            post(handlers::ssr::analysis::portfolio::actions::sync_inventory),
        )
        .route(
            "/analysis/resources",
            get(handlers::ssr::analysis::portfolio::page),
        )
        .route(
            "/analysis/resources.json",
            get(handlers::ssr::analysis::portfolio::json),
        )
        .route(
            "/analysis/resources/{entry_id}/bind",
            post(handlers::ssr::analysis::portfolio::actions::bind),
        )
        .route(
            "/analysis/versions",
            get(handlers::ssr::analysis::marketplace_versions::landing_page),
        )
        .route(
            "/analysis/versions/{marketplace_id}",
            get(handlers::ssr::analysis::marketplace_versions::detail_page),
        )
        .route(
            "/analysis/revisions/{id}",
            get(handlers::ssr::analysis::revisions::revision_page),
        )
        .route("/analysis", get(handlers::ssr::analysis::skills_page))
        .route(
            "/analysis/conversations",
            get(handlers::ssr::analysis::conversations::page),
        )
        .route(
            "/analysis/skills",
            get(handlers::ssr::analysis::skills_page),
        )
        .merge(evaluation_routes())
        .route(
            "/analysis/publications/review",
            post(handlers::ssr::analysis::lifecycle::review),
        )
        .route(
            "/analysis/publications/withdrawals",
            post(handlers::ssr::analysis::lifecycle::decide_withdrawal),
        )
}

// Why: the judge's pages are one feature; keeping them together also keeps
// `routes` inside the line budget the lint enforces.
fn evaluation_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/analysis/evaluation",
            get(handlers::ssr::analysis::evaluation::page),
        )
        .route(
            "/analysis/evaluation/runs",
            get(handlers::ssr::analysis::evaluation::runs_page),
        )
        .route(
            "/analysis/evaluation/runs/{id}",
            get(handlers::ssr::analysis::evaluation::run_page),
        )
        .route(
            "/analysis/evaluation/run",
            post(handlers::ssr::analysis::evaluation::actions::run_pending),
        )
        .route(
            "/analysis/evaluation/queue",
            post(handlers::ssr::analysis::evaluation::actions::queue_closed),
        )
}
