//! Per-user Atlassian (Jira + Confluence) OAuth 2.0 3LO linking, plus the typed
//! accessor that mints a per-user Atlassian bearer for the external MCP
//! servers.
//!
//! This is not a login. ADFS is the login; Atlassian is a downstream account a
//! signed-in user connects so that agent tool calls run with their own Jira and
//! Confluence permissions rather than a shared bot's.
//!
//! - [`atlassian_start`] / [`atlassian_callback`] drive the consent round trip.
//! - [`atlassian_token_handler`] is the accessor core's external-MCP client
//!   GETs to obtain a fresh bearer ([`crate::services::atlassian_oauth`]).
//! - [`atlassian_unlink`] drops the credential.
//!
//! One accessor serves both the Jira and Confluence MCP servers: it is keyed on
//! the calling user, and core passes it no server identity.

mod callback;
mod config;
mod start;
mod tokens;
mod unlink;

use std::sync::Arc;

use axum::http::HeaderMap;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use sqlx::PgPool;

use systemprompt::models::Config;

pub(crate) use callback::atlassian_callback;
pub use config::AtlassianConfig;
pub(crate) use config::{
    ACCESSIBLE_RESOURCES_URL, API_AUDIENCE, AUTHORIZE_URL, TOKEN_URL, client_secret,
};
pub(crate) use start::atlassian_start;
pub(crate) use tokens::atlassian_token_handler;
pub(crate) use unlink::atlassian_unlink;

pub(super) const STATE_COOKIE: &str = "atl_oauth_state";
const DEFAULT_REDIRECT: &str = "/admin/profile";

/// Errors from the Atlassian OAuth and token plumbing.
#[derive(Debug, thiserror::Error)]
pub enum AtlassianError {
    #[error("Atlassian HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Atlassian token endpoint returned {status}: {body}")]
    TokenEndpoint {
        status: reqwest::StatusCode,
        body: String,
    },
    // Why: distinct from a generic token-endpoint failure because it is the one
    // the caller can act on. Atlassian answers `invalid_grant` when the refresh
    // token is revoked, expired, or lost a rotation race; the only cure is to
    // reconnect, so the accessor deletes the row and reports "not linked"
    // rather than leaving the user on a permanent 502.
    #[error("Atlassian refresh token is no longer valid")]
    RefreshRejected,
    #[error("no Atlassian account linked for this user")]
    NotLinked,
    #[error("Atlassian is not configured")]
    NotConfigured,
    #[error("Atlassian credential store error: {0}")]
    Storage(#[from] systemprompt_web_shared::error::MarketplaceError),
    #[error("Atlassian credential crypto error: {0}")]
    Crypto(#[from] crate::repositories::secrets::secret_crypto::SecretCryptoError),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Per-request dependencies for the Atlassian handlers, shared via an axum
/// `Extension`.
#[derive(Clone)]
pub struct AtlassianDeps {
    pub config: Arc<AtlassianConfig>,
    // Why: the write pool, not the read pool. The mint path takes
    // `SELECT ... FOR UPDATE`, which on a read replica is at best an error and
    // at worst a lock that protects nothing.
    pub write_pool: Arc<PgPool>,
}

impl std::fmt::Debug for AtlassianDeps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AtlassianDeps")
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
                "config unavailable while setting state cookie; defaulting to Secure"
            );
            true
        },
    };
    if use_https { "; Secure" } else { "" }
}

// Why: reject anything that is not a same-site absolute path, to avoid turning
// the callback into an open redirect.
pub(super) fn sanitize_redirect(raw: Option<String>) -> String {
    match raw {
        Some(r) if r.starts_with('/') && !r.starts_with("//") => r,
        _ => DEFAULT_REDIRECT.to_owned(),
    }
}

// Why: 32 random bytes as base64url-no-pad (43 chars) — a valid PKCE verifier
// and a fine CSRF nonce.
pub(super) fn random_url_safe() -> String {
    let bytes: [u8; 32] = rand::rng().random();
    URL_SAFE_NO_PAD.encode(bytes)
}

// Why: parse the state cookie into `(state, code_verifier, redirect_to)`.
pub(super) fn read_state_cookie(headers: &HeaderMap) -> Option<(String, String, String)> {
    let raw = headers
        .get_all("cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|s| s.split(';'))
        .map(str::trim)
        .find_map(|kv| kv.strip_prefix(&format!("{STATE_COOKIE}=")))?;
    let mut parts = raw.splitn(3, '|');
    let state = parts.next()?.to_owned();
    let verifier = parts.next()?.to_owned();
    let redirect = parts.next()?.to_owned();
    Some((state, verifier, sanitize_redirect(Some(redirect))))
}
