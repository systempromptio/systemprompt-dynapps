//! JSON API, webhook, and secrets routes.

use std::sync::Arc;

use axum::Router;
use axum::http::HeaderMap;
use axum::routing::post;

use systemprompt::oauth::SessionCreationService;

use super::pools::DbHandles;
use super::{admin, farm_public, sign_out};
use crate::api;
use crate::extension::WebExtension;

pub(crate) fn build(db: &DbHandles, session_service: &Arc<SessionCreationService>) -> Router {
    let admin_api = admin::admin_router(
        Arc::clone(&db.read),
        &db.write,
        Arc::clone(&db.managed),
        Arc::clone(&db.farm),
    );
    let webhook_api =
        admin::hooks_webhook_router(&db.db, Arc::clone(&db.write), Arc::clone(session_service));
    let secrets_api = admin::secrets_router(Arc::clone(&db.write));
    let farm_api = farm_public::farm_public_router(Arc::clone(&db.read), Arc::clone(&db.farm));
    let bridge_identity =
        systemprompt_web_admin_platform::bridge_identity_router(Arc::clone(&db.read));
    let links_router = api::router(Arc::clone(&db.read), WebExtension::blog_config());

    Router::new()
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
            admin::service_accounts(&db.read, &db.write),
        ))
        .merge(webhook_api)
        .merge(secrets_api)
        .nest("/admin", admin_api)
        .nest(farm_public::FARM_PUBLIC_PREFIX, farm_api)
        .nest(
            farm_public::FARM_WORKER_PREFIX,
            farm_public::farm_worker_router(),
        )
}

pub(crate) fn share(db: &DbHandles) -> Router {
    admin::share_manifest_router(Arc::clone(&db.read))
}
