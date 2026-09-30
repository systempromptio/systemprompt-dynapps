//! Read-only administrator API routes.

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use sqlx::PgPool;

use crate::handlers;
use crate::routes::admin_groups;

pub(super) fn build_admin_read_routes_inner(read_pool: &Arc<PgPool>) -> Router {
    Router::new()
        .merge(systemprompt_web_admin_governance::routes::api::gateway_read_routes())
        .route("/users", get(handlers::list_users_handler))
        .route(
            "/users/{user_id}/detail",
            get(handlers::user_detail_handler),
        )
        .route("/users/{user_id}/usage", get(handlers::user_usage_handler))
        .route("/events", get(handlers::list_events_handler))
        .merge(systemprompt_web_admin_platform::routes::api::jobs_read_routes())
        .route(
            "/access-control",
            get(handlers::access_control::list_access_rules_handler),
        )
        .route(
            "/access-control/users/{user_id}/matrix",
            get(handlers::access_control::user_matrix_handler),
        )
        .merge(systemprompt_web_admin_platform::routes::api::sync_read_routes())
        .route(
            "/users/roles",
            get(handlers::gateway_access::list_distinct_roles_handler),
        )
        .route(
            "/users/{user_id}/roles",
            get(handlers::roles::get_user_roles_handler),
        )
        .route(
            "/users/{user_id}/scope-defaults",
            get(handlers::scope_defaults::get_user_scope_defaults_handler),
        )
        .merge(admin_groups::group_read_routes())
        .merge(admin_groups::project_read_routes())
        .route(
            "/users/search",
            get(handlers::gateway_access::search_users_handler),
        )
        .route(
            "/access-control/entity/{entity_type}/{entity_id}/access",
            get(handlers::entity_access::list_entity_access_handler),
        )
        .route(
            "/access-control/entity-access/all",
            get(handlers::entity_access::list_all_entity_access_handler),
        )
        .with_state(Arc::clone(read_pool))
}
