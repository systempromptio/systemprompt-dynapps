//! The authorization-code redemption at the tenant's token endpoint, and
//! where its client secret comes from.
//!
//! The secret is read per redemption, never captured at startup, so a secrets
//! reload (a rotated 1Password item) takes effect on the next sign-in.
//!
//! The form body is encoded by hand rather than through reqwest's optional
//! `form` feature. Response bodies are never logged: a 2xx one holds tokens,
//! and only the `error` code of a 4xx one is surfaced.

use std::fmt;

use serde::Deserialize;
use systemprompt::config::SecretsBootstrap;
use systemprompt::sso::SsoError;

/// Where the OIDC client secret is read from at each code redemption.
#[derive(Clone)]
pub enum ClientSecretSource {
    /// The named key in the secrets store, read afresh each time.
    Store(String),
    /// A fixed value, for tests against a fake `IdP`.
    Fixed(String),
}

impl ClientSecretSource {
    /// The secret as it stands now; `None` when absent or empty.
    #[must_use]
    pub fn current(&self) -> Option<String> {
        let secret = match self {
            Self::Store(key) => SecretsBootstrap::get().ok()?.get(key).cloned()?,
            Self::Fixed(secret) => secret.clone(),
        };
        Some(secret).filter(|secret| !secret.is_empty())
    }
}

impl fmt::Debug for ClientSecretSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Store(key) => f.debug_tuple("Store").field(key).finish(),
            Self::Fixed(_) => f.write_str("Fixed(..)"),
        }
    }
}

pub(super) struct CodeRedemption<'a> {
    pub token_endpoint: &'a str,
    pub code: &'a str,
    pub redirect_uri: &'a str,
    pub client_id: &'a str,
    pub client_secret: &'a str,
    pub code_verifier: &'a str,
}

#[derive(Deserialize)]
struct TokenResponse {
    id_token: Option<String>,
}

#[derive(Deserialize)]
struct TokenError {
    #[serde(default)]
    error: String,
}

fn form_body(pairs: &[(&str, &str)]) -> String {
    pairs
        .iter()
        .map(|(key, value)| format!("{key}={}", urlencoding::encode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

pub(super) async fn redeem_code(
    http: &reqwest::Client,
    req: &CodeRedemption<'_>,
) -> Result<String, SsoError> {
    let body = form_body(&[
        ("grant_type", "authorization_code"),
        ("code", req.code),
        ("redirect_uri", req.redirect_uri),
        ("client_id", req.client_id),
        ("client_secret", req.client_secret),
        ("code_verifier", req.code_verifier),
    ]);
    let response = http
        .post(req.token_endpoint)
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(body)
        .send()
        .await
        .map_err(|e| SsoError::Unavailable(format!("token endpoint unreachable: {e}")))?;

    let status = response.status();
    if status.is_server_error() {
        return Err(SsoError::Unavailable(format!(
            "token endpoint answered {status}"
        )));
    }
    if !status.is_success() {
        let code = response
            .json::<TokenError>()
            .await
            .map(|e| e.error)
            .unwrap_or_default();
        return Err(SsoError::InvalidAssertion(format!(
            "token endpoint refused the code ({status}): {code}"
        )));
    }
    response
        .json::<TokenResponse>()
        .await
        .map_err(|e| SsoError::InvalidAssertion(format!("token response unparseable: {e}")))?
        .id_token
        .ok_or_else(|| SsoError::InvalidAssertion("token response has no id_token".to_owned()))
}
