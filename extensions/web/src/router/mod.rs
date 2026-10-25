//! HTTP router assembly for the web extension.
//!
//! Split by concern: [`pools`] extracts the read/write database handles from
//! the type-erased extension context, [`admin`] composes the admin domain
//! crates' route fragments into the admin plane's routers, [`api`] mounts the
//! JSON/webhook plane, and [`admin_ssr`] mounts the server-rendered dashboard
//! plus the desktop-bridge sign-in flow, and [`sso`] discovers core's SSO
//! driver. `build` composes them over the extension's loaded config; if
//! the admin template engine cannot initialise, the API plane still mounts at
//! its normal prefix and only the SSR nests are skipped, so a misconfigured
//! deploy degrades loudly instead of moving routes.

pub(crate) mod admin;
mod admin_ssr;
mod api;
pub(crate) mod farm_public;
mod pools;
mod sign_out;
mod sso;

use std::sync::Arc;

use axum::{Extension, Router};

use systemprompt::extension::prelude::{ExtensionContext, ExtensionRouter};
use systemprompt::manifest::Config;
use systemprompt::security::keys::SigningKeys;

use crate::extension::WebExtension;
use pools::DbHandles;

pub(crate) fn build(ctx: &dyn ExtensionContext, web: &WebExtension) -> Option<ExtensionRouter> {
    let db = DbHandles::from_context(ctx, Arc::clone(&web.farm_runs))?;
    let signing_keys = signing_keys()?;
    let session_service = pools::build_session_service(&db, signing_keys.clone());

    let sso_deps = sso::build(&db, &session_service);

    let api_router = api::build(&db, &session_service, web.blog_config());
    let share_api = api::share(&db);

    let mut combined = Router::new()
        .merge(share_api)
        .nest("/api/public", api_router);

    match admin_ssr::build(&db, sso_deps, web.site.branding.clone()) {
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

    if let Some(core_planes) =
        systemprompt_web_admin_core::repositories::sync::core_planes::CorePlanes::from_context(
            ctx,
            systemprompt::scheduler::repository::ConfigReconcileRepository::new(&db.db),
        )
    {
        combined = combined.layer(Extension(core_planes));
    }

    Some(ExtensionRouter::public(
        combined
            .layer(Extension(Arc::clone(&web.site)))
            .layer(Extension(ctx.background_tasks()))
            .layer(Extension(signing_keys)),
        "/",
    ))
}

fn signing_keys() -> Option<SigningKeys> {
    let config = Config::get()
        .inspect_err(|error| tracing::error!(%error, "config unavailable; web routes not mounted"))
        .ok()?;
    SigningKeys::resolve(&config.signing_key_path)
        .inspect_err(
            |error| tracing::error!(%error, "signing keys unavailable; web routes not mounted"),
        )
        .ok()
}
