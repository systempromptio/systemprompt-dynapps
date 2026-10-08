//! Manifest sharing: the public manifest a share token unlocks. The admin
//! write that issues the token lives with the other admin API routes.

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use sqlx::PgPool;

use crate::handlers::share;

pub fn share_manifest_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route(
            "/share/manifest/{token}",
            get(share::public_manifest_handler),
        )
        .with_state(pool)
}
