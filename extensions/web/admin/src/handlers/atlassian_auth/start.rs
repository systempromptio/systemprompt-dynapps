//! `GET /admin/auth/atlassian/start` — redirect the browser to Atlassian's
//! authorize endpoint with PKCE and an anti-CSRF `state`.

use axum::Extension;
use axum::extract::Query;
use axum::http::HeaderMap;
use axum::http::header::SET_COOKIE;
use axum::response::{IntoResponse, Redirect, Response};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::{
    AUTHORIZE_URL, AtlassianDeps, STATE_COOKIE, random_url_safe, sanitize_redirect, secure_flag,
};
use crate::services::atlassian_oauth::audience_param;

#[derive(Deserialize)]
pub(crate) struct AtlassianStartParams {
    redirect: Option<String>,
}

fn connect_error(reason: &str) -> Response {
    // Why: lint-ok: http-error — this *is* the consent flow's failure channel: a
    // redirect back to the profile page carrying the reason, not an HTTP error.
    Redirect::to(&format!("/admin/profile?atlassian={reason}")).into_response()
}

pub(crate) async fn atlassian_start(
    Extension(deps): Extension<AtlassianDeps>,
    Query(params): Query<AtlassianStartParams>,
) -> Response {
    // Why: lint-ok: http-error — a consent flow reports failure by redirecting
    // back to the profile page with ?atlassian=<reason>; an error status would
    // strand the browser on a dead end instead of a usable page.
    let cfg = &deps.config;
    if !cfg.is_usable() {
        return connect_error("unavailable");
    }

    let state_token = random_url_safe();
    // Why: PKCE (RFC 7636) — a high-entropy verifier kept server-side and its
    // SHA-256 challenge sent to Atlassian, so an intercepted code is useless
    // without the verifier.
    let code_verifier = random_url_safe();
    let code_challenge = {
        let digest = Sha256::digest(code_verifier.as_bytes());
        URL_SAFE_NO_PAD.encode(digest)
    };
    let redirect_to = sanitize_redirect(params.redirect);

    // Why: the state cookie carries the CSRF nonce, the PKCE verifier and the
    // post-consent target, '|'-separated (base64url values and same-site paths
    // never contain '|').
    let cookie = format!(
        "{STATE_COOKIE}={state_token}|{code_verifier}|{redirect_to}; Path=/admin/auth/atlassian; HttpOnly; SameSite=Lax; Max-Age=600{}",
        secure_flag()
    );

    // Why: `prompt=consent` is not cosmetic — on a silent re-authorization
    // Atlassian omits the refresh token, and a credential without one dies in
    // an hour with no way to renew it.
    let authorize = format!(
        "{}?audience={}&response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256&prompt=consent",
        AUTHORIZE_URL,
        urlencoding::encode(audience_param()),
        urlencoding::encode(&cfg.consumer_key),
        urlencoding::encode(&cfg.redirect_uri),
        urlencoding::encode(&cfg.scopes),
        urlencoding::encode(&state_token),
        urlencoding::encode(&code_challenge),
    );

    let mut headers = HeaderMap::new();
    if let Ok(val) = cookie.parse() {
        headers.insert(SET_COOKIE, val);
    }
    (headers, Redirect::to(&authorize)).into_response()
}
