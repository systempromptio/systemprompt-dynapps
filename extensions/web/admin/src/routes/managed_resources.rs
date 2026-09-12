//! Managed authoring is independent of experiment execution and publication.

use crate::handlers::managed_resources as handlers;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::{Extension, Router};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::marketplace::managed::ManagedRepository;

pub(super) fn reads(pool: &Arc<PgPool>) -> Router {
    Router::new()
        .route("/managed/sources/{id}", get(handlers::source))
        .route("/managed/revisions/{id}", get(handlers::revision))
        .route("/managed/revisions/{id}/files", get(handlers::files))
        .layer(Extension(ManagedRepository::new((**pool).clone())))
}

pub(super) fn writes(pool: &Arc<PgPool>) -> Router {
    Router::new()
        .route("/managed/sources", post(handlers::create_source))
        .route(
            "/managed/sources/{id}/snapshots",
            post(handlers::capture_snapshot),
        )
        .route("/managed/resources", post(handlers::bind_resource))
        .route("/managed/revisions", post(handlers::create_revision))
        .layer(DefaultBodyLimit::max(32 * 1024 * 1024))
        .layer(Extension(ManagedRepository::new((**pool).clone())))
}
