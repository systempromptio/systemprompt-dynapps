//! Server-rendered routes for the bridge authentication flow.

use std::sync::Arc;

use axum::{Extension, Router, middleware as axum_middleware};
use sqlx::PgPool;
use tower_http::normalize_path::NormalizePathLayer;

use super::super::middleware;
use systemprompt_web_admin_core::templates::AdminTemplateEngine;

pub fn bridge_auth_ssr_router(pool: Arc<PgPool>, engine: AdminTemplateEngine) -> Router {
    let inner = systemprompt_web_admin_platform::routes::ssr_bridge::routes()
        .layer(Extension(engine))
        .layer(axum_middleware::from_fn_with_state(
            Arc::clone(&pool),
            middleware::marketplace_context_middleware,
        ))
        .layer(axum_middleware::from_fn(
            systemprompt_web_admin_core::middleware::gates::require_user_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            Arc::clone(&pool),
            middleware::user_context_middleware,
        ))
        .with_state(pool);

    Router::new().fallback_service(
        tower::ServiceBuilder::new()
            .layer(NormalizePathLayer::trim_trailing_slash())
            .service(inner),
    )
}
