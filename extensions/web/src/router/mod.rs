//! HTTP router assembly for the web extension.
//!
//! Split by concern: [`pools`] extracts the read/write database handles from
//! the type-erased extension context, [`admin`] composes the admin domain
//! crates' route fragments into the admin plane's routers, [`api`] mounts the
//! JSON/webhook plane, and [`admin_ssr`] mounts the server-rendered dashboard
//! plus the desktop-bridge sign-in flow, and [`sso`] installs the Entra ID
//! configs and discovers core's SSO driver over them. `build` composes them; if
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

use axum::{Extension, Router};

use systemprompt::extension::prelude::{ExtensionContext, ExtensionRouter};

use pools::DbHandles;

pub(crate) fn build(ctx: &dyn ExtensionContext) -> Option<ExtensionRouter> {
    let db = DbHandles::from_context(ctx)?;
    let session_service = pools::build_session_service(&db);

    let sso_deps = sso::build(&db, &session_service)?;

    let api_router = api::build(&db, &session_service);
    let share_api = api::share(&db);

    let mut combined = Router::new()
        .merge(share_api)
        .merge(
            systemprompt_web_admin_identity::routes::public::saml_metadata_routes()
                .layer(Extension(sso_deps.clone())),
        )
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
