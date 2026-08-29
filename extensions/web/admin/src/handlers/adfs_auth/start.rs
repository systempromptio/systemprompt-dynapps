//! `GET /admin/auth/adfs/start` — build an `AuthnRequest` and redirect the
//! browser to the AD FS SSO endpoint (HTTP-Redirect binding), stashing the
//! request id and an anti-CSRF `RelayState` in a cookie for the callback.

use std::time::{SystemTime, UNIX_EPOCH};

use axum::Extension;
use axum::extract::Query;
use axum::http::HeaderMap;
use axum::http::header::SET_COOKIE;
use axum::response::{IntoResponse, Redirect, Response};
use saml::{Binding, Dispatch, StartLogin};
use serde::Deserialize;

use super::provider::{idp_descriptor, service_provider};
use super::{AdfsDeps, STATE_COOKIE, login_error, random_url_safe, sanitize_redirect, secure_flag};

#[derive(Deserialize)]
pub(crate) struct StartParams {
    redirect: Option<String>,
}

pub(crate) async fn adfs_start(
    Extension(deps): Extension<AdfsDeps>,
    Query(params): Query<StartParams>,
) -> Response {
    // Why: lint-ok: http-error — an SSO flow reports failure by redirecting back to
    // the login page with ?sso=<reason>; an error status would strand the
    // browser on a dead end instead of returning the user to a usable page.
    let cfg = &deps.config;
    if !cfg.is_usable() {
        return login_error("unavailable");
    }
    let (idp, sp) = match (idp_descriptor(cfg).await, service_provider(cfg)) {
        (Ok(idp), Ok(sp)) => (idp, sp),
        (Err(e), _) | (_, Err(e)) => {
            tracing::error!(error = %e, "ADFS SSO cannot start");
            return login_error("unavailable");
        },
    };

    let state_token = random_url_safe();
    let started = match sp.start_login(
        &idp,
        StartLogin {
            relay_state: Some(&state_token),
            binding: Binding::HttpRedirect,
            force_authn: false,
            is_passive: false,
            requested_name_id_format: None,
            requested_authn_context: None,
            acs_index: None,
            acs_url: None,
            response_binding: None,
        },
    ) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "ADFS AuthnRequest could not be built");
            return login_error("unavailable");
        },
    };
    let Dispatch::Redirect(target) = started.dispatch else {
        tracing::error!("ADFS start produced a POST dispatch for a redirect binding");
        return login_error("unavailable");
    };

    let issued_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    let redirect_to = sanitize_redirect(params.redirect);

    // Why: The state cookie carries the RelayState nonce, the AuthnRequest id
    // the assertion must answer, its issue time, and the post-login target,
    // '|'-separated.
    let cookie = format!(
        "{STATE_COOKIE}={state_token}|{}|{issued_at}|{redirect_to}; Path=/admin/auth/adfs; HttpOnly; SameSite=Lax; Max-Age=600{}",
        started.tracker.request_id,
        secure_flag()
    );

    let mut headers = HeaderMap::new();
    if let Ok(val) = cookie.parse() {
        headers.insert(SET_COOKIE, val);
    }
    (headers, Redirect::to(target.as_str())).into_response()
}
