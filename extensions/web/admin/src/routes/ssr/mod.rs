//! Server-rendered admin page routes, grouped by dashboard section.
//!
//! Account routes are identity-scoped rather than admin-gated: each handler
//! resolves per request what the viewer may see, and 404s a conversation they
//! do not own. The AD FS and developer sign-in routes answer before any
//! session exists, beside the login page.

mod platform;

use std::sync::Arc;

use axum::routing::get;
use axum::{Extension, Router, middleware as axum_middleware};
use sqlx::PgPool;
use tower_http::normalize_path::NormalizePathLayer;

use super::super::{handlers, middleware};
use super::ssr_redirects;
use platform::platform_routes;
use systemprompt::database::DbPool;
use systemprompt_web_admin_analytics::routes::{activity_ssr, analysis_ssr, export_ssr, overview};
use systemprompt_web_admin_catalog::routes::ssr as catalog_ssr;
use systemprompt_web_admin_core::state::StateError;
use systemprompt_web_admin_core::templates::AdminTemplateEngine;
use systemprompt_web_admin_governance::routes::ssr_governance;
use systemprompt_web_admin_identity::AdfsDeps;
use systemprompt_web_admin_identity::routes::{account_ssr, people_ssr, public as sign_in};
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
    let inner = overview::routes()
        .merge(people_routes())
        .merge(ai_activity_routes())
        .merge(analysis_routes())
        .merge(governance_routes())
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
        .route("/login", get(handlers::login_page))
        .layer(axum_middleware::from_fn_with_state(
            pool,
            middleware::user_context_middleware,
        ));
    login.merge(sign_in::sign_in_routes())
}

fn ai_activity_routes() -> Router<Arc<PgPool>> {
    activity_ssr::routes()
        .merge(export_ssr::routes())
        .merge(ssr_governance::request_detail_routes())
}

fn analysis_routes() -> Router<Arc<PgPool>> {
    catalog_ssr::analysis_routes().merge(analysis_ssr::routes())
}

fn governance_routes() -> Router<Arc<PgPool>> {
    ssr_governance::routes().merge(overview::attention_routes())
}

fn people_routes() -> Router<Arc<PgPool>> {
    people_ssr::directory_routes()
        .merge(platform_ssr::device_routes())
        .merge(people_ssr::access_control_routes())
}

fn account_routes() -> Router<Arc<PgPool>> {
    account_ssr::profile_routes()
        .merge(activity_ssr::history_routes())
        .merge(platform_ssr::account_routes())
}

fn api_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/auth/me", get(middleware::auth_me_handler))
        .merge(ssr_governance::api_routes())
        .merge(activity_ssr::history_api_routes())
        .merge(overview::api_routes())
        .merge(account_ssr::profile_api_routes())
        .merge(platform_ssr::api_routes())
}
