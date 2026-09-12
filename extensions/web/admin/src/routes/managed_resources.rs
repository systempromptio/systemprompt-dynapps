//! Managed authoring is independent of experiment execution and publication.

use crate::handlers::managed_resources as handlers;
use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};

pub(super) fn reads() -> Router {
    Router::new()
        .route("/managed/sources/{id}", get(handlers::source))
        .route("/managed/revisions/{id}", get(handlers::revision))
        .route("/managed/revisions/{id}/files", get(handlers::files))
}

pub(super) fn writes() -> Router {
    Router::new()
        .route(
            "/managed/baselines/super-admin",
            post(crate::handlers::evaluation_baseline::capture_super_admin),
        )
        .route("/managed/sources", post(handlers::create_source))
        .route(
            "/managed/sources/{id}/snapshots",
            post(handlers::capture_snapshot),
        )
        .route("/managed/resources", post(handlers::bind_resource))
        .route("/managed/revisions", post(handlers::create_revision))
        .layer(DefaultBodyLimit::max(32 * 1024 * 1024))
}
