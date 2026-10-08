//! HTTP router assembly for the web extension.
//!
//! Split by concern: [`pools`] extracts the read/write database handles from
//! the type-erased extension context, [`admin`] composes the admin domain
//! crates' route fragments into the admin plane's routers, [`api`] mounts the
//! JSON/webhook plane, and [`admin_ssr`] mounts the server-rendered dashboard
//! plus the desktop-bridge sign-in flow. `build` composes them; if the admin
//! template engine cannot initialise, the API plane still mounts at its normal
//! prefix and only the SSR nests are skipped, so a misconfigured deploy
//! degrades loudly instead of moving routes.

pub(crate) mod admin;
mod admin_ssr;
mod api;
mod pools;
mod sign_out;

use std::sync::Arc;

use axum::Router;

use systemprompt::extension::prelude::{ExtensionContext, ExtensionRouter};

use pools::DbHandles;

pub(crate) fn build(ctx: &dyn ExtensionContext) -> Option<ExtensionRouter> {
    let db = DbHandles::from_context(ctx)?;
    let session_service = pools::build_session_service(&db);

    let sso_deps = systemprompt_web_admin_identity::AdfsDeps {
        config: crate::extension::WebExtension::adfs_config()
            .unwrap_or_else(|| Arc::new(systemprompt_web_admin_identity::AdfsConfig::disabled())),
        write_pool: Arc::clone(&db.write),
        session_service: Arc::clone(&session_service),
    };

    let api_router = api::build(&db, &session_service);
    let share_api = api::share(&db);

    let mut combined = Router::new()
        .merge(share_api)
        .nest("/api/public", api_router);

    match admin_ssr::build(&db, sso_deps) {
        Some(ssr) => {
            combined = Router::new()
                .nest_service("/admin", ssr.admin)
                .nest_service("/bridge-auth", ssr.bridge_auth)
                .merge(combined);
        },
        None => {
            tracing::error!(
                "Admin template engine failed to initialise; serving API routes without the SSR dashboard"
            );
        },
    }

    Some(ExtensionRouter::public(combined, "/"))
}
