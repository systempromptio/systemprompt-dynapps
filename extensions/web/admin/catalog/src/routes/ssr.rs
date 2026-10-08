//! Catalog page routes.
//!
//! They serve the installable units declared in `services/*.yaml`
//! and the retention lifecycle page in the Platform section, and the
//! marketplace-version, revision, inventory and publication routes in the
//! Analysis section.

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;

use crate::handlers;
use crate::handlers::ssr::analysis;

pub fn platform_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/mcp", get(handlers::catalog::mcp::mcp_servers_page))
        .route(
            "/mcp/{mcp_id}",
            get(handlers::catalog::mcp::mcp_detail_page),
        )
        .route(
            "/marketplaces",
            get(handlers::catalog::marketplaces::marketplaces_page),
        )
        .route(
            "/marketplaces/{marketplace_id}",
            get(handlers::catalog::marketplaces::marketplace_detail_page),
        )
        .route("/plugins", get(handlers::catalog::plugins_page))
        .route(
            "/plugins/{plugin_id}",
            get(handlers::catalog::plugin_detail_page),
        )
        .route("/skills", get(handlers::catalog::skills_page))
        .route(
            "/skills/{skill_id}",
            get(handlers::catalog::skill_detail_page),
        )
}

pub fn lifecycle_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/lifecycle", get(handlers::ssr::lifecycle_page))
        .route(
            "/lifecycle/archive/{tier}/{period}/{file}",
            get(handlers::ssr::lifecycle_archive_download),
        )
}

pub fn analysis_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/analysis/inventory/sync",
            post(analysis::inventory_actions::sync_inventory),
        )
        .route(
            "/analysis/inventory/{entry_id}/bind",
            post(analysis::inventory_actions::bind),
        )
        .route(
            "/analysis/versions",
            get(analysis::marketplace_versions::landing_page),
        )
        .route(
            "/analysis/versions/{marketplace_id}",
            get(analysis::marketplace_versions::detail_page),
        )
        .route(
            "/analysis/revisions/{id}",
            get(analysis::revisions::revision_page),
        )
        .route(
            "/analysis/publications/review",
            post(analysis::lifecycle::review),
        )
        .route(
            "/analysis/publications/withdrawals",
            post(analysis::lifecycle::decide_withdrawal),
        )
}
