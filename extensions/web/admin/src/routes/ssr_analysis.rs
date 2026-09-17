//! Analysis routes: conversations, skills, evaluation, marketplace versions
//! and the publication decisions taken from a marketplace's Distribution view.
//!
//! Every page is `/analysis/<noun>` with the entity's identity as the next
//! segment — the Platform section's `/<noun>/{id}` shape, one level in.

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
            "/analysis/skills",
            get(handlers::ssr::analysis::portfolio::page),
        )
        .route(
            "/analysis/skills.json",
            get(handlers::ssr::analysis::portfolio::json),
        )
        .route(
            "/analysis/skills/{skill}",
            get(handlers::ssr::analysis::skill_page),
        )
        .route(
            "/analysis/inventory/{entry_id}/bind",
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
        .route(
            "/analysis/conversations",
            get(handlers::ssr::analysis::conversations::page),
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
            get(handlers::ssr::analysis::evaluation::overview::page),
        )
        .route(
            "/analysis/evaluation/report",
            get(handlers::ssr::analysis::evaluation::reports::list_page)
                .post(handlers::ssr::analysis::evaluation::reports::regenerate),
        )
        .route(
            "/analysis/evaluation/report/{id}",
            get(handlers::ssr::analysis::evaluation::reports::detail_page),
        )
        .route(
            "/analysis/evaluation/{marketplace_id}",
            get(handlers::ssr::analysis::evaluation::marketplace::page),
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
