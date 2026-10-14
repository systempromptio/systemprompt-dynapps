//! Platform JSON API routes.
//!
//! The scheduled-jobs listing, the sync planes and the archive import, device
//! credential and service-account management, the per-user Salesforce link
//! and connector reprovisioning, and a person's own Odoo link (self-service).
//! The sync participant tier is its own fragment because the root gates it on
//! the scoped-console role rather than an admin tier; the archive import stays
//! with the manage-tier writes because a staged archive can rewrite every plane
//! at once.

use std::sync::Arc;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{delete, get, post, put};
use sqlx::PgPool;
use systemprompt_web_admin_core::repositories::sync::archive::MAX_UPLOAD_BYTES;

use crate::handlers;

pub fn jobs_read_routes() -> Router<Arc<PgPool>> {
    Router::new().route("/jobs", get(handlers::list_jobs_handler))
}

pub fn sync_read_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/sync/status", get(handlers::sync::status_handler))
        .route(
            "/sync/planes/{plane}/drift",
            get(handlers::sync::drift_handler),
        )
        .route(
            "/sync/planes/{plane}/export",
            get(handlers::sync::export_handler),
        )
        .route("/sync/export.zip", get(handlers::sync::export_zip_handler))
}

pub fn sync_write_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/sync/import",
            post(handlers::sync::stage_import_handler)
                .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES)),
        )
        .route(
            "/sync/import/{stage_id}/apply",
            post(handlers::sync::apply_import_handler),
        )
        .route(
            "/sync/import/{stage_id}",
            delete(handlers::sync::discard_import_handler),
        )
}

pub fn sync_participant_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/sync/planes/{plane}/apply",
            post(handlers::sync::apply_handler),
        )
        .route(
            "/sync/access-control/keep",
            post(handlers::sync::keep_handler),
        )
        .route(
            "/sync/sources/refresh",
            post(handlers::sync::refresh_sources_handler),
        )
}

pub fn connector_write_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/users/{user_id}/salesforce-identity",
            post(handlers::salesforce_identity::link_salesforce_identity_handler)
                .delete(handlers::salesforce_identity::unlink_salesforce_identity_handler),
        )
        .route(
            "/connectors/{provider}/reprovision",
            post(handlers::connector_reprovision::reprovision_connector_handler),
        )
}

pub fn service_account_write_routes() -> Router<Arc<PgPool>> {
    use handlers::service_accounts::api;
    Router::new()
        .route("/service-accounts", post(api::create))
        .route("/service-accounts/{user_id}", delete(api::delete))
        .route(
            "/service-accounts/{user_id}/credentials/{provider}",
            post(api::attach_credential).delete(api::remove_credential),
        )
}

pub fn odoo_link_routes() -> Router<Arc<PgPool>> {
    Router::new().route(
        "/user/odoo-link",
        post(handlers::odoo_link::link).delete(handlers::odoo_link::unlink),
    )
}

pub fn device_write_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/management/devices",
            post(handlers::devices::enroll_device),
        )
        .route(
            "/devices/{kind}/{id}",
            delete(handlers::devices::admin_revoke_credential),
        )
        .route(
            "/devices/certs/{id}/expiry",
            put(handlers::devices::admin_set_cert_expiry),
        )
}
