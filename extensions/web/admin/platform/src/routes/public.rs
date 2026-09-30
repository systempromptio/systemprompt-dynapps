//! The platform routers that mount outside the admin session.
//!
//! The desktop bridge's identity check, the Salesforce bearer accessor, and the
//! hosted-MCP connector consent and account API. Each authenticates its own
//! caller, so none sits behind the admin cookie layers or in the admin route
//! table.

use std::sync::Arc;

use axum::routing::{get, post};
use axum::{Extension, Router};
use sqlx::PgPool;

use crate::handlers;
use crate::handlers::connector_auth::{account, consent, identity};
use crate::handlers::salesforce_auth::SalesforceDeps;

// Why: the desktop bridge has no cookie jar, so this route authenticates the
// bridge token itself and must stay outside the admin session-cookie layers.
pub fn bridge_identity_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route(
            "/bridge/whoami",
            get(handlers::bridge_whoami::bridge_whoami_handler),
        )
        .with_state(pool)
}

// Why: core's external-MCP client GETs this with the user's bridge token and no
// session cookie, so it must not sit behind `require_auth_middleware`.
pub fn salesforce_api_router(deps: SalesforceDeps) -> Router {
    Router::new()
        .route(
            "/salesforce/token",
            get(handlers::salesforce_auth::salesforce_token_handler),
        )
        .layer(Extension(deps))
}

pub fn connector_api_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route("/account/connections", get(account::list))
        .route(
            "/account/connections/{provider}/disconnect",
            post(account::disconnect),
        )
        .route("/account/connections/{provider}/test", post(account::test))
        .route(
            "/account/connections/{provider}/manual-token",
            post(account::manual),
        )
        .route("/connectors/{provider}/start", get(consent::start))
        .route(
            "/connectors/salesforce/callback",
            get(consent::salesforce_callback),
        )
        .route("/connectors/{provider}/callback", get(consent::callback))
        .route("/connectors/{provider}/token", get(consent::token))
        .route("/identity/{server}/token", get(identity::token))
        .layer(axum::extract::DefaultBodyLimit::max(32768))
        .with_state(pool)
}
