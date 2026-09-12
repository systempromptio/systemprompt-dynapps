//! Analysis, source revision and candidate inspection routes.

use crate::handlers;
use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;
use std::sync::Arc;

pub(super) fn routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/analysis/evaluations/{id}",
            get(handlers::ssr::analysis::experiments::detail_page),
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
            post(handlers::evaluation_baseline::capture_super_admin_page),
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
            "/analysis/evaluations",
            get(handlers::ssr::analysis::experiments::list_page),
        )
}
