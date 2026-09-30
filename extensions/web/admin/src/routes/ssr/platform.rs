//! Platform page routes: the installable units declared in `services/*.yaml`,
//! the gateway that routes model traffic to their providers, and the
//! configuration, lifecycle, sync and observability pages. The gateway policy
//! editor writes `ai_gateway_policies`, which core re-reads per request, so its
//! saves are live edits rather than file edits.

use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use systemprompt_web_admin_analytics::routes::observability_ssr;
use systemprompt_web_admin_catalog::routes::ssr as catalog_routes;
use systemprompt_web_admin_governance::routes::ssr_governance as governance_routes;
use systemprompt_web_admin_platform::routes::platform_ssr as platform_ssr_routes;

pub(super) fn platform_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .merge(catalog_routes::platform_routes())
        .merge(governance_routes::gateway_routes())
        .merge(platform_ssr_routes::configuration_routes())
        .merge(catalog_routes::lifecycle_routes())
        .merge(platform_ssr_routes::sync_routes())
        .merge(observability_ssr::routes())
}
