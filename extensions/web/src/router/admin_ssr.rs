//! Server-rendered admin dashboard router.

use std::sync::Arc;

use axum::{Extension, Router};

use systemprompt_web_site::navigation::BrandingConfig;

use super::admin;
use super::pools::DbHandles;

pub(crate) struct SsrRouters {
    pub admin: Router,
    pub bridge_auth: Router,
}

pub(crate) fn build(
    db: &DbHandles,
    sso_deps: systemprompt_web_admin_identity::SsoDeps,
    branding: Option<BrandingConfig>,
) -> Option<SsrRouters> {
    let paths = super::pools::app_paths(&db.boot)?;
    let admin_dir = paths.storage().files().join("admin");
    let engine = systemprompt_web_admin_core::templates::AdminTemplateEngine::new(
        &admin_dir,
        paths.web().dist(),
    )
    .map_err(|e| tracing::error!(error = %e, "Failed to initialize admin template engine"))
    .ok()?
    .with_branding(branding);
    let admin = admin::admin_ssr_router(
        &db.db,
        Arc::clone(&db.read),
        admin::AdminSsrDeps {
            engine: engine.clone(),
            sso_deps,
            managed: Arc::clone(&db.managed),
            files: Arc::clone(&db.files),
            boot: db.boot.clone(),
            ai: Arc::clone(&db.ai),
        },
    )
    .layer(Extension(Arc::clone(&db.farm)));
    Some(SsrRouters {
        bridge_auth: admin::bridge_auth_ssr_router(Arc::clone(&db.read), engine),
        admin,
    })
}
