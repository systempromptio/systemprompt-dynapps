//! Read-only administrator API routes.

use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use systemprompt_web_admin_identity::routes::api as identity_routes;

pub(super) fn build_admin_read_routes_inner(read_pool: &Arc<PgPool>) -> Router {
    Router::new()
        .merge(systemprompt_web_admin_governance::routes::api::gateway_read_routes())
        .merge(identity_routes::user_read_routes())
        .merge(systemprompt_web_admin_analytics::routes::api::events_routes())
        .merge(systemprompt_web_admin_platform::routes::api::jobs_read_routes())
        .merge(identity_routes::access_read_routes())
        .merge(systemprompt_web_admin_platform::routes::api::sync_read_routes())
        .merge(identity_routes::directory_read_routes())
        .with_state(Arc::clone(read_pool))
}
