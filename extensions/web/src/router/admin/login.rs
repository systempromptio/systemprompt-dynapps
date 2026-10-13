//! The sign-in page, served before any session exists.
//!
//! A visitor who already has a session is sent on: to the console when their
//! roles grant it, otherwise to their profile.

use axum::Extension;
use axum::extract::Query;
use axum::response::{Html, IntoResponse, Redirect, Response};
use systemprompt_web_admin_core::error::AdminHtmlResult;
use systemprompt_web_admin_core::ssr::helpers::branding_context;
use systemprompt_web_admin_core::templates::AdminTemplateEngine;
use systemprompt_web_admin_core::types::UserContext;

#[derive(serde::Deserialize)]
pub(super) struct LoginParams {
    redirect: Option<String>,
}

#[derive(serde::Serialize)]
struct LoginContext<'a> {
    #[serde(flatten)]
    shell: systemprompt_web_admin_core::ssr::context::BrandingShell<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    redirect_encoded: Option<String>,
    sso_saml_enabled: bool,
    sso_oidc_enabled: bool,
    dev_login_enabled: bool,
}

pub(super) async fn login_page(
    user_ctx: Option<Extension<UserContext>>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(sso_deps): Extension<systemprompt_web_admin_identity::SsoDeps>,
    Query(params): Query<LoginParams>,
) -> AdminHtmlResult<Response> {
    if let Some(Extension(user_ctx)) = user_ctx {
        let target = if user_ctx.is_console {
            "/admin/"
        } else {
            "/admin/profile"
        };
        return Ok(Redirect::to(target).into_response());
    }

    let redirect_encoded = sanitize_login_redirect(params.redirect.as_deref())
        .map(|target| urlencoding::encode(&target).into_owned());

    let ctx = LoginContext {
        shell: branding_context(&engine),
        redirect_encoded,
        sso_saml_enabled: sso_deps
            .provider_enabled(systemprompt_web_admin_identity::sso::entra_saml::ID),
        sso_oidc_enabled: sso_deps
            .provider_enabled(systemprompt_web_admin_identity::sso::entra_oidc::ID),
        dev_login_enabled: systemprompt_web_admin_identity::dev_login_enabled(),
    };
    let html = engine.render("login", &ctx)?;
    Ok(Html(html).into_response())
}

fn sanitize_login_redirect(raw: Option<&str>) -> Option<String> {
    let raw = raw?.trim();
    (raw.starts_with('/') && !raw.starts_with("//")).then(|| raw.to_owned())
}
