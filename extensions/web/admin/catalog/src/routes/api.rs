//! Admin API routes the catalog owns: the plugin and agent listings, and the
//! write that issues a manifest share token.

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;

use crate::handlers;

pub fn listing_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/plugins", get(handlers::plugins::list_plugins_handler))
        .route("/agents", get(handlers::resources::list_agents_handler))
        .route(
            "/agents/{agent_id}",
            get(handlers::resources::get_agent_handler),
        )
}

pub fn share_token_routes() -> Router<Arc<PgPool>> {
    Router::new().route(
        "/users/{user_id}/share-token",
        post(handlers::share::issue_share_token_handler),
    )
}
