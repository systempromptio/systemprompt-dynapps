//! Admin JSON API routes.
//!
//! Read and write routes are built against separate pools so read traffic can
//! be directed at a replica. That split is also the authorisation boundary,
//! and it has three tiers: reads open to any console role, writes to the two
//! admin roles, and a platform tier holding the controls that decide what the
//! directory itself grants. One shared admin layer would have been simpler,
//! but every write here — user create/update/delete, session and identity
//! mutation, device enrolment, access-control rules, gateway configuration —
//! relies on that layer for its only authorisation check, so widening it to
//! the semi-admin role would have handed a project manager the whole control
//! plane. The boundary lives in the router where it can be read off in one
//! place rather than in thirty handler bodies.
//!
//! Two narrower tiers sit beside those. A participant tier admits any
//! scoped-console caller to the plane apply and the source refresh; the
//! handlers narrow a non-admin's apply to their own marketplaces. The archive
//! import stays in the manage tier because a staged archive can rewrite every
//! plane at once. A self-service tier carries the two writes that touch only
//! the caller's own account, targeted from the validated session, so they are
//! not admin-gated. Approval verdicts are one route per verb, so a mistyped
//! verdict 404s at the router before any row is touched.

use std::sync::Arc;

use axum::routing::{delete, get, patch, post, put};
use axum::{Router, middleware as axum_middleware};
use sqlx::PgPool;

use super::super::handlers;
use super::admin_groups;
use systemprompt::database::DbPool;
use systemprompt_web_admin_catalog::routes as catalog_routes;
use systemprompt_web_admin_core::state::StateError;
use systemprompt_web_admin_core::types::{ROLES_CONSOLE, ROLES_MANAGE, ROLES_PLATFORM};
use systemprompt_web_admin_governance::routes::api as governance_routes;
use systemprompt_web_admin_platform::routes::api as platform_routes;

mod read;
use read::build_admin_read_routes_inner;

pub(crate) fn build_admin_only_routes(
    db: &DbPool,
    read_pool: &Arc<PgPool>,
    write_pool: &Arc<PgPool>,
    owner: systemprompt::identifiers::UserId,
) -> Result<Router, StateError> {
    let reads = build_admin_read_routes_inner(read_pool)
        .merge(catalog_routes::managed_resources::reads())
        .layer(axum_middleware::from_fn_with_state(
            ROLES_CONSOLE,
            systemprompt_web_admin_core::middleware::gates::require_roles_middleware,
        ));
    let writes = build_admin_write_routes(write_pool)
        .merge(catalog_routes::managed_resources::writes())
        .layer(axum_middleware::from_fn_with_state(
            ROLES_MANAGE,
            systemprompt_web_admin_core::middleware::gates::require_roles_middleware,
        ));
    let platform =
        build_admin_platform_routes(write_pool).layer(axum_middleware::from_fn_with_state(
            ROLES_PLATFORM,
            systemprompt_web_admin_core::middleware::gates::require_roles_middleware,
        ));
    let participant = platform_routes::sync_participant_routes()
        .with_state(Arc::clone(write_pool))
        .layer(axum_middleware::from_fn(
            systemprompt_web_admin_core::middleware::gates::require_scoped_console_middleware,
        ));

    let managed = Arc::new(systemprompt_web_admin_core::state::ManagedState::new(
        db, owner,
    )?);
    Ok(reads
        .merge(writes)
        .merge(platform)
        .merge(participant)
        .layer(axum::Extension(managed)))
}

fn build_access_control_write_routes() -> Router<Arc<PgPool>> {
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

fn build_admin_write_routes(write_pool: &Arc<PgPool>) -> Router {
    Router::new()
        .merge(governance_routes::gateway_write_routes())
        .route(
            "/users",
            post(handlers::users_bootstrap::create_user_handler),
        )
        .route(
            "/users/{user_id}",
            put(handlers::update_user_handler).delete(handlers::delete_user_handler),
        )
        .merge(catalog_routes::api::share_token_routes())
        .merge(platform_routes::connector_write_routes())
        .route(
            "/users/{user_id}/sessions",
            get(handlers::list_user_sessions_handler)
                .delete(handlers::revoke_all_user_sessions_handler),
        )
        .route(
            "/users/{user_id}/sessions/{session_id}",
            delete(handlers::revoke_user_session_handler),
        )
        .merge(build_access_control_write_routes())
        .merge(platform_routes::sync_write_routes())
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
        .merge(platform_routes::device_write_routes())
        .merge(governance_routes::approval_write_routes())
        .with_state(Arc::clone(write_pool))
}

fn build_admin_platform_routes(write_pool: &Arc<PgPool>) -> Router {
    Router::new()
        .merge(admin_groups::group_platform_routes())
        .merge(admin_groups::project_platform_routes())
        .with_state(Arc::clone(write_pool))
}

pub(crate) fn build_self_service_routes(write_pool: &Arc<PgPool>) -> Router {
    Router::new()
        .route(
            "/user/settings",
            put(handlers::self_service::update_own_settings_handler),
        )
        .route(
            "/user/account",
            delete(handlers::self_service::delete_own_account_handler),
        )
        .with_state(Arc::clone(write_pool))
}

pub(crate) fn build_auth_read_routes(read_pool: &Arc<PgPool>) -> Router {
    Router::new()
        .route("/dashboard", get(handlers::dashboard_handler))
        .merge(catalog_routes::api::listing_routes())
        .merge(governance_routes::plugin_env_routes())
        .with_state(Arc::clone(read_pool))
}
