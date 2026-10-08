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
//! [`session`] (the session JWT mint and cookie shared by every browser
//! sign-in, the development login link included).

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

use systemprompt::manifest::Config;
use systemprompt::oauth::SessionCreationService;

pub(crate) use callback::adfs_callback;
pub use config::{
    ACS_PATH as ADFS_ACS_PATH, AdfsConfig, METADATA_PATH as ADFS_METADATA_PATH,
    group_matches_pattern,
};
pub use identity::AssertionClaims;
pub use session::permissions_for_roles;
pub(crate) use session::{SessionSubject, mint_session, record_login, session_cookie};
pub(crate) use start::adfs_start;

pub(super) const STATE_COOKIE: &str = "adfs_saml_state";
const STATE_COOKIE_SCOPE: &str = "Path=/admin/auth/adfs; HttpOnly";
const DEFAULT_REDIRECT: &str = "/admin";

/// SAML plumbing errors: logged once at the HTTP boundary, surfaced to the
/// browser only as an opaque `?sso=<reason>`.
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

pub(crate) fn use_https() -> bool {
    match Config::get() {
        Ok(c) => c.use_https,
        Err(e) => {
            tracing::error!(
                error = %e,
                "config unavailable while setting session cookie; defaulting to Secure — the cookie will be dropped by browsers on plain http"
            );
            true
        },
    }
}

pub(crate) fn secure_flag() -> &'static str {
    if use_https() { "; Secure" } else { "" }
}

pub(super) fn sanitize_redirect(raw: Option<String>) -> String {
    match raw {
        Some(r) if r.starts_with('/') && !r.starts_with("//") => r,
        _ => DEFAULT_REDIRECT.to_owned(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LoginRedirect(pub(super) &'static str);

impl IntoResponse for LoginRedirect {
    fn into_response(self) -> Response {
        Redirect::to(&format!("/admin/login?sso={}", self.0)).into_response()
    }
}

pub(super) fn random_url_safe() -> String {
    let bytes: [u8; 32] = rand::rng().random();
    URL_SAFE_NO_PAD.encode(bytes)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowState {
    pub state: String,
    pub request_id: String,
    pub issued_at_unix: u64,
    pub redirect_to: String,
}

pub fn read_state_cookie(headers: &HeaderMap) -> Option<FlowState> {
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

// Why: AD FS posts the ACS cross-site and browsers withhold `SameSite=Lax`
// cookies on that POST, so the `saml` crate would reject `InResponseTo` with no
// tracker. `None` needs `Secure`, hence `Lax` on plain-http local dev only.
const fn same_site_attrs(secure: bool) -> &'static str {
    if secure {
        "SameSite=None; Secure"
    } else {
        "SameSite=Lax"
    }
}

#[must_use]
pub fn state_cookie(flow: &FlowState, secure: bool) -> String {
    format!(
        "{STATE_COOKIE}={}|{}|{}|{}; {STATE_COOKIE_SCOPE}; {}; Max-Age=600",
        flow.state,
        flow.request_id,
        flow.issued_at_unix,
        flow.redirect_to,
        same_site_attrs(secure)
    )
}

#[must_use]
pub fn clear_state_cookie(secure: bool) -> String {
    format!(
        "{STATE_COOKIE}=; {STATE_COOKIE_SCOPE}; {}; Max-Age=0",
        same_site_attrs(secure)
    )
}
