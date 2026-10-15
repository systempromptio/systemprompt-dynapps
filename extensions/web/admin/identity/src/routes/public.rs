//! Sign-in routes that answer before any session exists.
//!
//! The Entra ID `OpenID Connect` sign-in at `/auth/entra/*`, served by core's
//! SSO driver, and the developer login link. The link is mounted only on a
//! development non-cloud profile, so production has no route to misconfigure
//! open. Every handler reads [`crate::SsoDeps`] from an `Extension` the
//! composition root layers on.

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use sqlx::PgPool;

use crate::handlers;
use crate::handlers::sso_alias;

pub fn sign_in_routes() -> Router<Arc<PgPool>> {
    let routes = Router::new()
        .route("/auth/entra/start", get(sso_alias::oidc_start))
        .route("/auth/entra/callback", get(sso_alias::oidc_callback));
    dev_login_routes(routes)
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
