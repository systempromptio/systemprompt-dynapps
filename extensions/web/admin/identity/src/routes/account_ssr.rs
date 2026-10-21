//! Account routes.
//!
//! The viewer's own profile page, the connect code it mints for the
//! desktop bridge, and the relay to the viewer's personal-subscription keys.
//! Identity-scoped rather than admin-gated — each handler reads only the
//! signed-in user's own account.

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post, put};
use sqlx::PgPool;

use crate::handlers;

pub fn profile_routes() -> Router<Arc<PgPool>> {
    Router::new().route("/profile", get(handlers::ssr::profile_page))
}

pub fn profile_api_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/api/profile/bridge-code",
            post(handlers::ssr::issue_bridge_code),
        )
        .route(
            "/api/profile/provider-credentials",
            get(handlers::provider_credentials::list_own_provider_credentials),
        )
        .route(
            "/api/profile/provider-credentials/{provider}",
            put(handlers::provider_credentials::put_own_provider_credential)
                .delete(handlers::provider_credentials::delete_own_provider_credential),
        )
}
