//! The authorization request: the redirect URL with PKCE, state and nonce,
//! and the RFC 9207 check on the `iss` the response comes back with.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;
use sha2::{Digest, Sha256};
use systemprompt::sso::SsoReauth;

use super::verify::resolve_issuer_template;

/// The parameters of one authorization request.
#[derive(Debug, Clone, Copy)]
pub struct AuthorizeRequest<'a> {
    pub authorization_endpoint: &'a str,
    pub client_id: &'a str,
    pub redirect_uri: &'a str,
    pub scopes: &'a [String],
    pub state: &'a str,
    pub nonce: &'a str,
    pub code_challenge: &'a str,
    pub reauth: SsoReauth,
}

#[must_use]
pub fn build_authorize_url(req: &AuthorizeRequest<'_>) -> String {
    let scope = req.scopes.join(" ");
    let max_age = req
        .reauth
        .max_age
        .filter(|max_age| *max_age >= 0)
        .map(|max_age| max_age.to_string());
    let prompt = if req.reauth.force {
        "login"
    } else {
        "select_account"
    };
    let mut params = vec![
        ("client_id", req.client_id),
        ("response_type", "code"),
        ("redirect_uri", req.redirect_uri),
        ("response_mode", "query"),
        ("scope", scope.as_str()),
        ("state", req.state),
        ("nonce", req.nonce),
        ("code_challenge", req.code_challenge),
        ("code_challenge_method", "S256"),
        ("prompt", prompt),
    ];
    if let Some(max_age) = max_age.as_deref() {
        params.push(("max_age", max_age));
    }
    let query = params
        .iter()
        .map(|(key, value)| format!("{key}={}", urlencoding::encode(value)))
        .collect::<Vec<_>>()
        .join("&");
    let separator = if req.authorization_endpoint.contains('?') {
        '&'
    } else {
        '?'
    };
    format!("{}{separator}{query}", req.authorization_endpoint)
}

// Why: RFC 9207 §2.4 — an `iss` on the authorization response must name the
// issuer the request went to; its absence is allowed, since Entra ID does not
// advertise `authorization_response_iss_parameter_supported`.
#[must_use]
pub fn authorization_response_issuer_matches(
    iss: &str,
    discovered_issuer: &str,
    configured_issuer: &str,
    tenant: &str,
) -> bool {
    iss == resolve_issuer_template(discovered_issuer, tenant) || iss == configured_issuer
}

#[must_use]
pub fn pkce_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

pub(super) fn random_b64url<const N: usize>() -> String {
    let mut bytes = [0u8; N];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}
