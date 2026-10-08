//! Identity API fragments.
//!
//! Users and their sessions, roles and scope defaults, groups and projects,
//! and the access-control rules.
//!
//! The admin composition root merges each fragment at the position and
//! behind the role tier it has always had — reads for the console roles,
//! writes for the two admin roles, the AD-mapping routes for
//! `platform_admin`, and the two self-service writes for any signed-in user.

use std::sync::Arc;

use axum::Router;
use axum::routing::{delete, get, patch, post, put};
use sqlx::PgPool;

use super::admin_groups;
use crate::handlers;

pub fn user_read_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/users", get(handlers::list_users_handler))
        .route(
            "/users/{user_id}/detail",
            get(handlers::user_detail_handler),
        )
        .route("/users/{user_id}/usage", get(handlers::user_usage_handler))
}

pub fn access_read_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/access-control",
            get(handlers::access_control::list_access_rules_handler),
        )
        .route(
            "/access-control/users/{user_id}/matrix",
            get(handlers::access_control::user_matrix_handler),
        )
}

pub fn directory_read_routes() -> Router<Arc<PgPool>> {
    Router::new()
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
}

pub fn user_write_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/users",
            post(handlers::users_bootstrap::create_user_handler),
        )
        .route(
            "/users/{user_id}",
            put(handlers::update_user_handler).delete(handlers::delete_user_handler),
        )
}

pub fn session_write_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/users/{user_id}/sessions",
            get(handlers::list_user_sessions_handler)
                .delete(handlers::revoke_all_user_sessions_handler),
        )
        .route(
            "/users/{user_id}/sessions/{session_id}",
            delete(handlers::revoke_user_session_handler),
        )
}

pub fn access_control_write_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/access-control/entity/{entity_type}/{entity_id}",
            put(handlers::access_control::update_entity_rules_handler),
        )
        .route(
            "/access-control/bulk",
            put(handlers::access_control::bulk_assign_handler),
        )
        .route(
            "/access-control/entity/{entity_type}/{entity_id}/rules",
            post(handlers::entity_access::upsert_entity_rule_handler),
        )
        .route(
            "/access-control/entity/{entity_type}/{entity_id}/rules/{rule_id}",
            delete(handlers::entity_access::delete_entity_rule_handler),
        )
        .route(
            "/access-control/entity/{entity_type}/{entity_id}/default",
            patch(handlers::entity_access::set_entity_default_handler),
        )
        .route(
            "/access-control/bulk-template",
            post(handlers::entity_access::apply_template_handler),
        )
}

pub fn directory_write_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/users/{user_id}/roles",
            put(handlers::roles::set_user_roles_handler),
        )
        .route(
            "/users/{user_id}/scope-defaults",
            put(handlers::scope_defaults::set_user_scope_defaults_handler),
        )
        .route(
            "/scope-defaults/recompute",
            post(handlers::scope_defaults::recompute_scope_defaults_handler),
        )
        .merge(admin_groups::group_write_routes())
        .merge(admin_groups::project_write_routes())
}

pub fn directory_platform_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .merge(admin_groups::group_platform_routes())
        .merge(admin_groups::project_platform_routes())
}

pub fn self_service_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/user/settings",
            put(handlers::self_service::update_own_settings_handler),
        )
        .route(
            "/user/account",
            delete(handlers::self_service::delete_own_account_handler),
        )
}
