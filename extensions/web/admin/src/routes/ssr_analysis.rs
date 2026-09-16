//! Analysis, source revision and candidate inspection routes.

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
            "/analysis/revisions/{id}/edit",
            get(handlers::ssr::analysis::candidates::edit_page)
                .post(handlers::ssr::analysis::candidates::save_candidate)
                .layer(axum::extract::DefaultBodyLimit::max(4 * 1024 * 1024)),
        )
        .route(
            "/analysis/revisions/{baseline}/compare/{candidate}",
            get(handlers::ssr::analysis::candidates::comparison_page),
        )
        .route(
            "/analysis/versions",
            get(handlers::ssr::analysis::versions::resources_page),
        )
        .route(
            "/analysis/versions/capture-baseline",
            post(handlers::managed_resources::capture_super_admin_page),
        )
        .route(
            "/analysis/versions/generations/{baseline}/compare/{candidate}",
            get(handlers::ssr::analysis::versions::compare_page),
        )
        .route(
            "/analysis/versions/{id}",
            get(handlers::ssr::analysis::versions::resource_page),
        )
        .route(
            "/analysis/revisions/{id}",
            get(handlers::ssr::analysis::versions::revision_page),
        )
        .route("/analysis", get(handlers::ssr::analysis::skills_page))
        .route(
            "/analysis/skills",
            get(handlers::ssr::analysis::skills_page),
        )
        .route(
            "/analysis/impact",
            get(handlers::ssr::analysis::impact::page),
        )
        .route(
            "/analysis/publications",
            get(handlers::ssr::analysis::lifecycle::page),
        )
        .route(
            "/analysis/publications/review",
            post(handlers::ssr::analysis::lifecycle::review),
        )
        .route(
            "/analysis/publications/withdrawals",
            post(handlers::ssr::analysis::lifecycle::decide_withdrawal),
        )
}
