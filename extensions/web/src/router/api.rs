//! JSON API, webhook, and secrets routes.

use std::sync::Arc;

use axum::Router;
use axum::http::HeaderMap;
use axum::routing::post;

use systemprompt::oauth::SessionCreationService;

use super::pools::DbHandles;
use super::sign_out;
use crate::extension::WebExtension;
use crate::{admin, api};

pub(crate) fn build(
    db: &DbHandles,
    session_service: &Arc<SessionCreationService>,
) -> Result<Router, systemprompt_web_admin_core::state::StateError> {
    let admin_api = admin::admin_router(&db.db, Arc::clone(&db.read), &db.write, db.owner.clone())?;
    let webhook_api =
        admin::hooks_webhook_router(&db.db, Arc::clone(&db.write), Arc::clone(session_service))?;
    let secrets_api = admin::secrets_router(Arc::clone(&db.write));
    let bridge_identity =
        systemprompt_web_admin_platform::bridge_identity_router(Arc::clone(&db.read));
    let links_router = api::router(Arc::clone(&db.read), WebExtension::blog_config());
    let salesforce_api = systemprompt_web_admin_platform::salesforce_api_router(
        systemprompt_web_admin_platform::SalesforceDeps {
            config: WebExtension::salesforce_config().unwrap_or_else(|| {
                Arc::new(systemprompt_web_admin_platform::SalesforceConfig::disabled())
            }),
            write_pool: Arc::clone(&db.write),
        },
    );

    Ok(Router::new()
        .route(
            "/auth/session",
            post(api::auth::set_session).delete({
                let pool = Arc::clone(&db.write);
                move |headers: HeaderMap| sign_out::sign_out(Arc::clone(&pool), headers)
            }),
        )
        .merge(links_router)
        .merge(bridge_identity)
        .merge(systemprompt_web_admin_platform::connector_api_router(
            Arc::clone(&db.write),
        ))
        .merge(salesforce_api)
        .merge(webhook_api)
        .merge(secrets_api)
        .nest("/admin", admin_api))
}

pub(crate) fn share(db: &DbHandles) -> Router {
    admin::share_manifest_router(Arc::clone(&db.read))
}
