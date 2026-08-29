//! "Sign in with Astound SSO" — SAML 2.0 Web Browser SSO against the
//! corporate AD FS farm, with this platform as the relying party.
//!
//! - [`adfs_start`] redirects the browser to AD FS with a signed-shape
//!   `AuthnRequest` (HTTP-Redirect binding) and an anti-CSRF `RelayState`.
//! - [`adfs_callback`] is the assertion consumer service: it verifies the
//!   posted `SAMLResponse` (XML-DSig against the metadata-pinned certificate,
//!   audience, destination, validity window, replay), maps the AD group
//!   attribute to roles, resolves the identity, and mints the session.
//!
//! Module layout: [`config`] (the loaded YAML), [`provider`] (the SP and `IdP`
//! descriptors built from it), [`start`] (the redirect), [`callback`] (the
//! ACS handler), [`identity`] (claim gating + federated resolution),
//! [`session`] (the session JWT mint shared by any browser SSO).

mod callback;
mod config;
mod identity;
mod provider;
mod session;
mod start;

use std::sync::Arc;

use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use sqlx::PgPool;

use systemprompt::models::Config;
use systemprompt::oauth::SessionCreationService;

pub(crate) use callback::adfs_callback;
pub use config::AdfsConfig;
pub use identity::AssertionClaims;
pub(crate) use start::adfs_start;

pub(super) const STATE_COOKIE: &str = "adfs_saml_state";
const DEFAULT_REDIRECT: &str = "/admin";

// Why: Errors from the SAML plumbing. Logged once at the HTTP boundary; the
// browser only ever sees an opaque `?sso=<reason>`.
#[derive(Debug, thiserror::Error)]
pub enum AdfsError {
    #[error("IdP federation metadata is unusable: {0}")]
    Metadata(String),
    #[error("service-provider configuration rejected: {0}")]
    ServiceProvider(String),
    #[error("SAMLResponse is not valid base64: {0}")]
    Base64(String),
    #[error("assertion rejected: {0}")]
    Assertion(#[source] saml::Error),
}

/// Per-request dependencies for the ADFS handlers, shared via an axum
/// `Extension`.
#[derive(Clone)]
pub struct AdfsDeps {
    pub config: Arc<AdfsConfig>,
    pub write_pool: Arc<PgPool>,
    pub session_service: Arc<SessionCreationService>,
}

impl std::fmt::Debug for AdfsDeps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdfsDeps")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

pub(super) fn secure_flag() -> &'static str {
    let use_https = match Config::get() {
        Ok(c) => c.use_https,
        Err(e) => {
            tracing::error!(
                error = %e,
                "config unavailable while setting session cookie; defaulting to Secure — the cookie will be dropped by browsers on plain http"
            );
            true
        },
    };
    if use_https { "; Secure" } else { "" }
}

// Why: Reject anything that isn't a same-site absolute path, to avoid
// open-redirect.
pub(super) fn sanitize_redirect(raw: Option<String>) -> String {
    match raw {
        Some(r) if r.starts_with('/') && !r.starts_with("//") => r,
        _ => DEFAULT_REDIRECT.to_owned(),
    }
}

// Why: lint-ok: http-error — this *is* the SSO failure channel: a redirect back
// to the login page carrying the reason, not an HTTP error.
pub(super) fn login_error(reason: &str) -> Response {
    Redirect::to(&format!("/admin/login?sso={reason}")).into_response()
}

// Why: 32 random bytes as base64url-no-pad (43 chars) — a fine CSRF nonce for
// `RelayState`.
pub(super) fn random_url_safe() -> String {
    let bytes: [u8; 32] = rand::rng().random();
    URL_SAFE_NO_PAD.encode(bytes)
}

// Why: What the start handler stashed and the callback reads back: the
// RelayState nonce, the AuthnRequest id the assertion must answer, when it
// was issued, and the post-login target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FlowState {
    pub state: String,
    pub request_id: String,
    pub issued_at_unix: u64,
    pub redirect_to: String,
}

// Why: Parse the state cookie. The segments are base64url values, a SAML id,
// a number, and a same-site path — none contain '|'.
pub(super) fn read_state_cookie(headers: &HeaderMap) -> Option<FlowState> {
    let raw = headers
        .get_all("cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|s| s.split(';'))
        .map(str::trim)
        .find_map(|kv| kv.strip_prefix(&format!("{STATE_COOKIE}=")))?;
    let mut parts = raw.splitn(4, '|');
    let state = parts.next()?.to_owned();
    let request_id = parts.next()?.to_owned();
    let issued_at_unix = parts.next()?.parse().ok()?;
    let redirect_to = sanitize_redirect(parts.next().map(ToOwned::to_owned));
    Some(FlowState {
        state,
        request_id,
        issued_at_unix,
        redirect_to,
    })
}
