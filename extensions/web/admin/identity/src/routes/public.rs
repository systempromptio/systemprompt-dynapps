//! Sign-in routes that answer before any session exists.
//!
//! The AD FS redirect and assertion consumer service, and the developer login
//! link. The link is
//! mounted only on a development non-cloud profile, so production has no
//! route to misconfigure open.

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;

use crate::handlers;

pub fn sign_in_routes() -> Router<Arc<PgPool>> {
    let routes = Router::new()
        .route("/auth/adfs/start", get(handlers::adfs_auth::adfs_start))
        .route("/auth/adfs/acs", post(handlers::adfs_auth::adfs_callback));
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
