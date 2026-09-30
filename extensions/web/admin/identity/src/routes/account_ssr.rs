//! Account routes.
//!
//! The viewer's own profile page and the connect code it mints for the
//! desktop bridge. Identity-scoped rather than admin-gated —
//! each handler reads only the signed-in user's own account.

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;

use crate::handlers;

pub fn profile_routes() -> Router<Arc<PgPool>> {
    Router::new().route("/profile", get(handlers::ssr::profile_page))
}

pub fn profile_api_routes() -> Router<Arc<PgPool>> {
    Router::new().route(
        "/api/profile/bridge-code",
        post(handlers::ssr::issue_bridge_code),
    )
}
