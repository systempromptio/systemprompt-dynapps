//! Sign-in routes that answer before any session exists.
//!
//! The Entra ID sign-ins — SAML at the historical `/auth/adfs/*` paths, OIDC
//! at `/auth/entra/*` — served by core's SSO driver, the SAML metadata
//! document, and the developer login link. The link is mounted only on a
//! development non-cloud profile, so production has no route to misconfigure
//! open. Every handler reads [`crate::SsoDeps`] from an `Extension` the
//! composition root layers on.

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;

use crate::handlers;
use crate::handlers::sso_alias;

pub fn sign_in_routes() -> Router<Arc<PgPool>> {
    let routes = Router::new()
        .route("/auth/adfs/start", get(sso_alias::saml_start))
        .route("/auth/adfs/acs", post(sso_alias::saml_acs))
        .route("/auth/entra/start", get(sso_alias::oidc_start))
        .route("/auth/entra/callback", get(sso_alias::oidc_callback));
    dev_login_routes(routes)
}

pub fn saml_metadata_routes() -> Router {
    Router::new().route(
        crate::sso::entra_saml::METADATA_PATH,
        get(sso_alias::saml_metadata),
    )
}

fn dev_login_routes(router: Router<Arc<PgPool>>) -> Router<Arc<PgPool>> {
    if !handlers::dev_login::dev_login_enabled() {
        return router;
    }
    router.route(
        "/auth/dev/login",
        get(handlers::dev_login::dev_login_redeem),
    )
}
