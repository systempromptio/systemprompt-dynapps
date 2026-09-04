//! Admin JSON API routes.
//!
//! Read and write routes are built against separate pools so read traffic can
//! be directed at a replica. That split is also the authorisation boundary:
//! reads sit behind `require_console_middleware` (`admin` or
//! `project_manager`), writes behind `require_admin_middleware` (`admin`
//! alone). One shared admin layer would have been simpler, but every write
//! here — user create/update/delete, session and identity mutation, device
//! enrolment, access-control rules, gateway configuration — relies on that
//! layer for its only authorisation check, so widening it to the semi-admin
//! role would have handed a project manager the whole control plane. The
//! boundary lives in the router where it can be read off in one place rather
//! than in thirty handler bodies.

use std::sync::Arc;

use axum::routing::{get, patch, post, put};
use axum::{Router, middleware as axum_middleware};
use sqlx::PgPool;

use super::super::{handlers, middleware};

pub(crate) fn build_admin_only_routes(read_pool: &Arc<PgPool>, write_pool: &Arc<PgPool>) -> Router {
    // Why: the split is the `project_manager` boundary. Reads are the admin
    // dashboard's data and open to any console role; every write here mutates
    // an identity, a role, an ACL rule or the gateway config, so it stays
    // `admin` only.
    let reads = build_admin_read_routes_inner(read_pool).layer(axum_middleware::from_fn(
        middleware::require_console_middleware,
    ));
    let writes = build_admin_write_routes(write_pool).layer(axum_middleware::from_fn(
        middleware::require_admin_middleware,
    ));

    reads.merge(writes)
}

fn build_admin_read_routes_inner(read_pool: &Arc<PgPool>) -> Router {
    Router::new()
        .route("/gateway", get(handlers::get_gateway_handler))
        .route(
            "/gateway/catalog/for-user/{user_id}",
            get(handlers::gateway_catalog::for_user_handler),
        )
        .route(
            "/gateway/acl/detect",
            get(handlers::gateway_catalog::detect_handler),
        )
        .route("/users", get(handlers::list_users_handler))
        .route(
            "/users/{user_id}/detail",
            get(handlers::user_detail_handler),
        )
        .route("/users/{user_id}/usage", get(handlers::user_usage_handler))
        .route("/events", get(handlers::list_events_handler))
        .route("/jobs", get(handlers::list_jobs_handler))
        .route(
            "/access-control",
            get(handlers::access_control::list_access_rules_handler),
        )
        .route(
            "/access-control/projects",
            get(handlers::access_control::access_control_projects_handler),
        )
        .route(
            "/access-control/users/{user_id}/matrix",
            get(handlers::access_control::user_matrix_handler),
        )
        .route(
            "/access-control/yaml-snapshot",
            get(handlers::access_control::yaml_snapshot_handler),
        )
        .route(
            "/users/roles",
            get(handlers::gateway_access::list_distinct_roles_handler),
        )
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

fn build_admin_write_routes(write_pool: &Arc<PgPool>) -> Router {
    Router::new()
        .route("/gateway", patch(handlers::update_gateway_settings_handler))
        .route(
            "/gateway/routes",
            post(handlers::create_gateway_route_handler),
        )
        .route(
            "/gateway/routes/{idx}",
            patch(handlers::update_gateway_route_handler)
                .delete(handlers::delete_gateway_route_handler),
        )
        .route(
            "/gateway/routes/reorder",
            post(handlers::reorder_gateway_routes_handler),
        )
        .route(
            "/users",
            post(handlers::users_bootstrap::create_user_handler),
        )
        .route(
            "/users/{user_id}",
            put(handlers::update_user_handler).delete(handlers::delete_user_handler),
        )
        .route(
            "/users/{user_id}/share-token",
            post(handlers::share::issue_share_token_handler),
        )
        .route(
            "/users/{user_id}/slack-identity",
            post(handlers::slack_identity::link_slack_identity_handler)
                .delete(handlers::slack_identity::unlink_slack_identity_handler),
        )
        // Why: set administratively because ADFS replaced the Salesforce SSO
        // login that used to capture the Username from a `preferred_username`
        // claim. Nothing derives it now, so someone has to state it.
        .route(
            "/users/{user_id}/salesforce-identity",
            post(handlers::salesforce_identity::link_salesforce_identity_handler)
                .delete(handlers::salesforce_identity::unlink_salesforce_identity_handler),
        )
        .route(
            "/users/{user_id}/sessions",
            get(handlers::list_user_sessions_handler)
                .delete(handlers::revoke_all_user_sessions_handler),
        )
        .route(
            "/users/{user_id}/sessions/{session_id}",
            axum::routing::delete(handlers::revoke_user_session_handler),
        )
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
            axum::routing::delete(handlers::entity_access::delete_entity_rule_handler),
        )
        .route(
            "/access-control/entity/{entity_type}/{entity_id}/default",
            patch(handlers::entity_access::set_entity_default_handler),
        )
        .route(
            "/access-control/bulk-template",
            post(handlers::entity_access::apply_template_handler),
        )
        .merge(build_management_write_routes())
        .with_state(Arc::clone(write_pool))
}

// Why: the management surface (devices) is split out so the
// gateway/user/access-control routes above stay readable as one list; both
// halves are mounted on the same write pool. A user's project is not settable
// here: it is the AD group, re-projected at every sign-in.
fn build_management_write_routes() -> Router<Arc<PgPool>> {
    Router::new().route(
        "/management/devices",
        post(handlers::devices::enroll_device),
    )
}

pub(crate) fn build_auth_read_routes(read_pool: &Arc<PgPool>) -> Router {
    Router::new()
        .route("/dashboard", get(handlers::dashboard_handler))
        .route("/plugins", get(handlers::list_plugins_handler))
        .route(
            "/plugins/{plugin_id}/env",
            get(handlers::list_plugin_env_handler),
        )
        .route("/agents", get(handlers::list_agents_handler))
        .route("/agents/{agent_id}", get(handlers::get_agent_handler))
        .with_state(Arc::clone(read_pool))
}
