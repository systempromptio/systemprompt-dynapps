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
use systemprompt_web_admin_governance::routes::ssr_governance as governance_routes;

pub(super) fn platform_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .merge(catalog_routes::platform_routes())
        .merge(governance_routes::gateway_routes())
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
