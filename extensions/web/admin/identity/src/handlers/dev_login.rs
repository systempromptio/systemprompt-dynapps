//! `GET /admin/auth/dev/login?code=…` — redeem a one-shot developer login
//! code for an `access_token` session cookie.
//!
//! The route exists only on a development, non-cloud profile: it is not
//! refused elsewhere, it is never mounted, so production has no path to hit.
//! Every failure is the same redirect to `/admin/login?dev=invalid` — a
//! missing, unknown, spent or expired code, or a mint error — so no outcome
//! says whether an account exists. "Spent" allows a short grace after the
//! first redeem, because an address-bar prefetch reaches the link before the
//! person does. The code itself is issued by the
//! `dev-login` CLI extension, which applies the same gate before it writes.

use axum::Extension;
use axum::extract::Query;
use axum::extract::rejection::QueryRejection;
use axum::http::HeaderMap;
use axum::http::header::SET_COOKIE;
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use systemprompt::manifest::Profile;
use systemprompt::manifest::profile::{Environment, ProfileType};

use systemprompt::analytics::SessionAnalyticsBuilder;
use systemprompt::sso::{BrowserSessionSubject, mint_browser_session};
use systemprompt_web_admin_core::activity::NewActivity;
use systemprompt_web_admin_core::repositories::user_activity::record;

use super::sso_alias::SsoDeps;
use crate::repositories::dev_login::consume_dev_login_code;

pub const DEV_LOGIN_PATH: &str = "/admin/auth/dev/login";
const INVALID_REDIRECT: &str = "/admin/login?dev=invalid";

pub const fn dev_login_allowed(environment: Environment, target: ProfileType) -> bool {
    environment.is_development() && !target.is_cloud()
}

#[must_use]
pub const fn dev_login_enabled(profile: &Profile) -> bool {
    dev_login_allowed(profile.runtime.environment, profile.target)
}

pub fn dev_login_url(api_external_url: &str, code: &str) -> String {
    format!(
        "{}{DEV_LOGIN_PATH}?code={code}",
        api_external_url.trim_end_matches('/')
    )
}

#[derive(Debug, Deserialize)]
pub(crate) struct DevLoginParams {
    code: Option<String>,
}

pub(crate) async fn dev_login_redeem(
    Extension(deps): Extension<SsoDeps>,
    Extension(boot): Extension<systemprompt_web_admin_core::extension_boot::ExtensionBoot>,
    headers: HeaderMap,
    params: Result<Query<DevLoginParams>, QueryRejection>,
) -> Result<Response, DevLoginRejected> {
    let Ok(Query(DevLoginParams { code: Some(code) })) = params else {
        return Err(DevLoginRejected);
    };

    let user = match consume_dev_login_code(&deps.write_pool, &code).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(DevLoginRejected),
        Err(e) => {
            tracing::error!(error = %e, "dev login code lookup failed");
            return Err(DevLoginRejected);
        },
    };

    let config = boot.config();
    let subject = BrowserSessionSubject {
        user_id: &user.user_id,
        display_name: &user.display_name,
        email: &user.email,
        roles: &user.roles,
    };
    let analytics = SessionAnalyticsBuilder::new(&headers).build();
    let minted = match mint_browser_session(&deps.session_service, config, &subject, &analytics)
        .await
    {
        Ok(minted) => minted,
        Err(e) => {
            tracing::error!(error = %e, user_id = %user.user_id, "dev login session mint failed");
            return Err(DevLoginRejected);
        },
    };
    record(
        &deps.write_pool,
        NewActivity::login(&user.user_id, &user.display_name),
    )
    .await;

    tracing::info!(user_id = %user.user_id, email = %user.email, "dev login code redeemed");

    let mut out = HeaderMap::new();
    if let Ok(val) =
        session_cookie(&minted.access_token, minted.max_age_secs, config.use_https).parse()
    {
        out.append(SET_COOKIE, val);
    }
    Ok((out, Redirect::to("/admin")).into_response())
}

fn session_cookie(jwt: &str, max_age: i64, secure: bool) -> String {
    let secure = if secure { "; Secure" } else { "" };
    format!("access_token={jwt}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}{secure}")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DevLoginRejected;

impl IntoResponse for DevLoginRejected {
    fn into_response(self) -> Response {
        Redirect::to(INVALID_REDIRECT).into_response()
    }
}
