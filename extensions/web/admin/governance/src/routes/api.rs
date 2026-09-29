//! Governance JSON API routes.
//!
//! They cover the gateway configuration and ACL catalog, the approval verdicts
//! and per-plugin environment variables. Approval verdicts
//! are one route per verb, so a mistyped verdict 404s at the router before any
//! row is touched.

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, patch, post};
use sqlx::PgPool;

use crate::handlers;

pub fn gateway_read_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/gateway", get(handlers::gateway::get_gateway_handler))
        .route(
            "/gateway/catalog/for-user/{user_id}",
            get(handlers::gateway_catalog::for_user_handler),
        )
        .route(
            "/gateway/acl/detect",
            get(handlers::gateway_catalog::detect_handler),
        )
}

pub fn gateway_write_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/gateway",
            patch(handlers::gateway::update_gateway_settings_handler),
        )
        .route(
            "/gateway/routes",
            post(handlers::gateway::create_gateway_route_handler),
        )
        .route(
            "/gateway/routes/{idx}",
            patch(handlers::gateway::update_gateway_route_handler)
                .delete(handlers::gateway::delete_gateway_route_handler),
        )
        .route(
            "/gateway/routes/reorder",
            post(handlers::gateway::reorder_gateway_routes_handler),
        )
}

pub fn approval_write_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/approvals/{call_id}/approve",
            post(handlers::approvals::approve_handler),
        )
        .route(
            "/approvals/{call_id}/deny",
            post(handlers::approvals::deny_handler),
        )
}

pub fn plugin_env_routes() -> Router<Arc<PgPool>> {
    Router::new().route(
        "/plugins/{plugin_id}/env",
        get(handlers::plugins_env::list_plugin_env_handler),
    )
}
