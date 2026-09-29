//! The governance webhooks Claude Code and MCP servers call before a tool runs.
//!
//! `/hooks/govern` gives a hook-driven verdict and `/govern/authz` the entity
//! authorization check. The composition root merges this fragment
//! beside `/hooks/track` and layers the shared extensions over both.

use std::sync::Arc;

use axum::Router;
use axum::routing::post;
use sqlx::PgPool;

use crate::handlers;

pub fn govern_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/hooks/govern", post(handlers::webhook::govern_tool_use))
        .route("/govern/authz", post(handlers::webhook::govern_authz))
}
