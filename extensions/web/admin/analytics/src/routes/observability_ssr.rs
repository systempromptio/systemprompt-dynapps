//! The observability page under the Platform section: the exporter's state,
//! an export on demand, and a connection test against the configured
//! endpoint.

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;

use crate::handlers;

pub fn routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/system/observability",
            get(handlers::ssr::observability_page),
        )
        .route(
            "/system/observability/export",
            post(handlers::ssr::observability_export_now),
        )
        .route(
            "/system/observability/test",
            post(handlers::ssr::observability_test_connection),
        )
}
