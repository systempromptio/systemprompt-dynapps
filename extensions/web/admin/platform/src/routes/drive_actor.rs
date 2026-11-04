//! The document actor for the `/drive` routes, with core's browser-session
//! rules.
//!
//! These routes sit behind this install's own session middleware, not core's
//! auth layer, so no `RequestContext` reaches them and core's `actor_scope`
//! would leave every call unscoped. This layer applies the same rules from
//! the token itself: the person is a token core signed `browser_session`
//! and that arrived as the `access_token` cookie with no `Authorization`
//! header. A cookie carrying any other token sends the caller back to sign
//! in (`reauth`): a page load gets the cookie cleared and a trip to sign-in
//! that comes back here; a script call gets `401` with `error_key: reauth`,
//! which the page's transport turns into the same trip. There is no grace
//! period for a session minted before the claim existed. A browser
//! session acts only from this site's pages (core's `same_origin`), except
//! a top-level `GET` navigation, so a link from mail still opens a document;
//! anything else cross-site is refused `403`. A bearer call keeps core's
//! default, an unnamed unattended agent.
//!
//! The site's origins are derived as core's `actor_scope` derives them: the
//! public URL and the advertised one, so a proxy that rewrites `Host` cannot
//! cause a false refusal.

use axum::extract::Request;
use axum::http::header::{ACCEPT, AUTHORIZATION, COOKIE, SET_COOKIE};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Extension, Json};
use serde::Serialize;
use systemprompt::api::routes::documents::actor_scope::same_origin;
use systemprompt::files::documents::agents::{DocumentActor, act_as};
use systemprompt::models::auth::{JwtAudience, JwtClaims};
use systemprompt::oauth::validate_jwt_token;
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;

const SESSION_COOKIE: &str = "access_token=";
const SIGN_IN: &str = "/admin/login";
pub const REAUTH_KEY: &str = "reauth";

fn reauth(boot: &ExtensionBoot, request: &Request) -> Response {
    let secure = if boot.config().use_https {
        "; Secure"
    } else {
        ""
    };
    let clear = format!("access_token=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{secure}");
    if wants_html(request.headers()) {
        let back = request
            .uri()
            .path_and_query()
            .map_or_else(|| request.uri().path().to_owned(), ToString::to_string);
        let target = format!("{SIGN_IN}?redirect={}", urlencoding::encode(&back));
        return ([(SET_COOKIE, clear)], Redirect::to(&target)).into_response();
    }
    let body = ReauthBody {
        error: "your session predates this sign-in; sign in again",
        error_key: REAUTH_KEY,
    };
    (StatusCode::UNAUTHORIZED, [(SET_COOKIE, clear)], Json(body)).into_response()
}

#[derive(Debug, Serialize)]
struct ReauthBody {
    error: &'static str,
    error_key: &'static str,
}

#[must_use]
pub fn wants_html(headers: &HeaderMap) -> bool {
    headers
        .get(ACCEPT)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|accept| accept.contains("text/html"))
}

#[must_use]
pub fn cookie_token(headers: &HeaderMap) -> Option<String> {
    if headers.contains_key(AUTHORIZATION) {
        return None;
    }
    headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .find_map(|c| c.trim().strip_prefix(SESSION_COOKIE))
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
}

#[must_use]
pub const fn is_browser_session(claims: &JwtClaims) -> bool {
    claims.custom.browser_session
}

fn site_origins(boot: &ExtensionBoot) -> Vec<String> {
    let config = boot.config();
    [
        Some(&config.api_external_url),
        config.advertise_url.as_ref(),
    ]
    .into_iter()
    .flatten()
    .filter_map(|raw| url::Url::parse(raw).ok())
    .map(|url| url.origin().ascii_serialization())
    .filter(|origin| origin != "null")
    .collect()
}

#[must_use]
pub fn may_act_from(method: &Method, headers: &HeaderMap, site_origins: &[String]) -> bool {
    let navigation = headers
        .get("sec-fetch-mode")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|mode| mode == "navigate");
    same_origin(headers, site_origins) || (*method == Method::GET && navigation)
}

pub(super) async fn person_actor_middleware(
    Extension(boot): Extension<ExtensionBoot>,
    request: Request,
    next: Next,
) -> Response {
    let Some(token) = cookie_token(request.headers()) else {
        return next.run(request).await;
    };
    let claims = validate_jwt_token(boot.token_authority().as_ref(), &token, &[JwtAudience::Api]);
    if let Err(error) = &claims {
        tracing::debug!(%error, "drive session cookie did not validate");
    }
    if !claims.as_ref().is_ok_and(is_browser_session) {
        return reauth(&boot, &request);
    }
    if !may_act_from(
        request.method(),
        request.headers(),
        &site_origins(&boot),
    ) {
        return (
            StatusCode::FORBIDDEN,
            "a browser session acts only from this site's pages",
        )
            .into_response();
    }
    act_as(DocumentActor::Human, next.run(request)).await
}
