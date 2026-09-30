//! Server-rendered admin page routes, grouped by dashboard section.
//!
//! The developer login link is mounted only on a development non-cloud
//! profile, so production has no route to misconfigure open. Account routes
//! are identity-scoped rather than admin-gated: each handler resolves per
//! request what the viewer may see, and 404s a conversation they do not own.

mod activity;
mod platform;

use std::sync::Arc;

use axum::routing::{get, post};
use axum::{Extension, Router, middleware as axum_middleware};
use sqlx::PgPool;
use tower_http::normalize_path::NormalizePathLayer;

use super::super::{handlers, middleware};
use super::ssr_redirects;
use crate::handlers::adfs_auth::AdfsDeps;
use activity::ai_activity_routes;
use platform::platform_routes;
use systemprompt::database::DbPool;
use systemprompt_web_admin_core::state::StateError;
use systemprompt_web_admin_core::templates::AdminTemplateEngine;
use systemprompt_web_admin_platform::routes::platform_ssr;

pub fn admin_ssr_router(
    db: &DbPool,
    pool: Arc<PgPool>,
    engine: AdminTemplateEngine,
    sso_deps: AdfsDeps,
    owner: systemprompt::identifiers::UserId,
) -> Result<Router, StateError> {
    let managed = Arc::new(systemprompt_web_admin_core::state::ManagedState::new(
        db, owner,
    )?);
    let inner = overview_routes()
        .merge(people_routes())
        .merge(ai_activity_routes())
        .merge(super::ssr_analysis::routes())
        .merge(super::ssr_governance::routes())
        .merge(platform_routes())
        .merge(account_routes())
        .merge(api_routes())
        .merge(ssr_redirects::legacy_routes())
        .layer(Extension(engine.clone()))
        .layer(Extension(managed))
        .layer(Extension(sso_deps.clone()))
        .layer(axum_middleware::from_fn_with_state(
            Arc::clone(&pool),
            middleware::marketplace_context_middleware,
        ))
        .layer(axum_middleware::from_fn(
            systemprompt_web_admin_core::middleware::gates::non_admin_gate_middleware,
        ))
        .layer(axum_middleware::from_fn(
            systemprompt_web_admin_core::middleware::gates::require_user_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            Arc::clone(&pool),
            middleware::user_context_middleware,
        ))
        .with_state(Arc::clone(&pool));

    let combined = public_routes(Arc::clone(&pool))
        .layer(Extension(engine))
        .layer(Extension(sso_deps))
        .with_state(pool)
        .fallback_service(inner);

    Ok(Router::new().fallback_service(
        tower::ServiceBuilder::new()
            .layer(NormalizePathLayer::trim_trailing_slash())
            .service(combined),
    ))
}

fn public_routes(pool: Arc<PgPool>) -> Router<Arc<PgPool>> {
    let login = Router::new()
        .route("/login", get(handlers::ssr::login_page))
        .layer(axum_middleware::from_fn_with_state(
            pool,
            middleware::user_context_middleware,
        ));
    let routes = login
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

fn overview_routes() -> Router<Arc<PgPool>> {
    Router::new().route("/", get(handlers::ssr::overview_page))
}

fn people_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/users", get(handlers::ssr::users_page))
        .route(
            "/users/{user_id}",
            get(handlers::ssr::user_detail_by_id_page),
        )
        .route("/user", get(handlers::ssr::user_detail_page))
        .route("/groups", get(handlers::ssr::groups_page))
        .route("/groups/{group_id}", get(handlers::ssr::group_detail_page))
        .route("/projects", get(handlers::ssr::projects_page))
        .route(
            "/projects/{project_id}",
            get(handlers::ssr::project_detail_page),
        )
        .route(
            "/projects/{project_id}/report",
            get(handlers::ssr::project_report_page),
        )
        .route("/roles", get(handlers::ssr::roles_page))
        .merge(platform_ssr::device_routes())
        .route("/access-control", get(handlers::ssr::access_control_page))
}

fn account_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/profile", get(handlers::ssr::profile_page))
        .route("/history", get(handlers::ssr::history_page))
        .route(
            "/history/conversations/{context_id}",
            get(handlers::ssr::history_conversation_page),
        )
        .merge(platform_ssr::account_routes())
}

fn api_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/auth/me", get(middleware::auth_me_handler))
        .merge(systemprompt_web_admin_governance::routes::ssr_governance::api_routes())
        .route("/api/history/search", get(handlers::ssr::history_search))
        .route("/api/overview", get(handlers::ssr::overview_fragment))
        .route(
            "/api/profile/bridge-code",
            post(handlers::ssr::issue_bridge_code),
        )
        .merge(platform_ssr::api_routes())
}
