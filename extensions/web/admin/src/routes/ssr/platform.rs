//! Platform page routes: the installable units declared in `services/*.yaml`,
//! the gateway that routes model traffic to their providers, and the
//! configuration, lifecycle, sync and observability pages. The gateway policy
//! editor writes `ai_gateway_policies`, which core re-reads per request, so its
//! saves are live edits rather than file edits.

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;

use crate::handlers;
use systemprompt_web_admin_catalog::routes::ssr as catalog_routes;

pub(super) fn platform_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .merge(catalog_routes::platform_routes())
        .route("/gateway", get(handlers::ssr::gateway_page))
        .route(
            "/gateway/routes/{route_id}",
            get(handlers::ssr::gateway_route_page),
        )
        .route(
            "/gateway/policies",
            get(handlers::ssr::gateway_policies_page).post(handlers::ssr::save_gateway_policy),
        )
        .route(
            "/gateway/policies/delete",
            post(handlers::ssr::delete_gateway_policy),
        )
        .route("/configuration", get(handlers::ssr::configuration_page))
        .merge(catalog_routes::lifecycle_routes())
        .route("/sync", get(handlers::ssr::sync_page))
        .route(
            "/sync/import/{stage_id}",
            get(handlers::ssr::import_preview_page),
        )
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
