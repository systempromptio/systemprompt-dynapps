//! Server-rendered platform pages.
//!
//! The configuration, code-sync and drive pages in the Platform group, the
//! device credentials and service-account pages in People, and the connect,
//! connectors and setup pages every signed-in user reaches from their account.

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;

use crate::handlers;

pub fn configuration_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/configuration", get(handlers::ssr::configuration_page))
        .route(
            "/platform/secrets/reload",
            post(handlers::ssr::secrets_reload),
        )
}

pub fn sync_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/sync", get(handlers::ssr::sync_page))
        .route(
            "/sync/import/{stage_id}",
            get(handlers::ssr::import_preview_page),
        )
}

pub fn device_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/devices", get(handlers::ssr::devices_page))
        .route("/devices/pats", post(handlers::devices::issue_pat))
        .route(
            "/devices/pats/{id}",
            axum::routing::delete(handlers::devices::revoke_pat),
        )
        .route(
            "/devices/certs/{id}",
            axum::routing::delete(handlers::devices::revoke_cert),
        )
}

pub fn drive_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/drive", get(handlers::ssr::drive_page))
        .route(
            "/drive/{document_id}",
            get(handlers::ssr::drive_document_page),
        )
}

pub fn service_account_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/identity/service-accounts",
            get(handlers::service_accounts::service_accounts_page),
        )
        .route(
            "/identity/service-accounts/{user_id}",
            get(handlers::service_accounts::service_account_page),
        )
}

pub fn account_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/connect", get(handlers::ssr::connect_page))
        .route("/connectors", get(handlers::ssr::connectors_page))
        .route("/setup", get(handlers::ssr::setup_page))
}

// Why: Instance-wide operational data; handlers enforce platform-admin access.
pub fn data_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/logs", get(handlers::ssr::data_console::logs_page))
        .route("/sql", get(handlers::ssr::data_console::sql_page))
        .route("/sql/query", post(handlers::ssr::data_console::run_query))
        .route(
            "/data/clear/preview",
            post(handlers::ssr::data_console::clear_preview),
        )
        .route(
            "/data/clear/confirm",
            post(handlers::ssr::data_console::clear_confirm),
        )
}
