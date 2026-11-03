//! The authorize request's parameters and PKCE (RFC 7636, S256 only).
//!
//! Every parameter is checked before the session is looked at, so a malformed
//! request never reaches the login redirect or a consent page. The redirect
//! URI is a constant: an authorize request naming any other is refused, and
//! the relay never redirects anywhere a request chose. PKCE verifiers and
//! `state` are RFC 3986 unreserved characters; the S256 transform is
//! `BASE64URL(SHA256(ASCII(verifier)))`.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::REDIRECT_URI;
use crate::http::pairings::device_name;

const CHALLENGE_LEN: usize = 43;
const STATE_MIN: usize = 32;
const STATE_MAX: usize = 256;
const VERIFIER_MIN: usize = 43;
const VERIFIER_MAX: usize = 128;

#[derive(Debug, Default, Deserialize)]
pub struct AuthorizeQuery {
    pub redirect_uri: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub state: Option<String>,
    pub device_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizeRequest {
    pub challenge: String,
    pub state: String,
    pub device_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ParamError {
    #[error("redirect_uri must be dynwork://auth/callback")]
    RedirectUri,
    #[error("code_challenge must be a 43-character base64url S256 challenge")]
    Challenge,
    #[error("code_challenge_method must be S256")]
    Method,
    #[error("state must be 32 to 256 unreserved characters")]
    State,
    #[error("device_name must be at most 64 printable characters")]
    DeviceName,
}

const fn unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~')
}

fn base64url(text: &str) -> bool {
    text.bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

pub fn validate(query: AuthorizeQuery) -> Result<AuthorizeRequest, ParamError> {
    if query.redirect_uri.as_deref() != Some(REDIRECT_URI) {
        return Err(ParamError::RedirectUri);
    }
    if query
        .code_challenge_method
        .as_deref()
        .is_some_and(|method| method != "S256")
    {
        return Err(ParamError::Method);
    }
    let challenge = query
        .code_challenge
        .filter(|challenge| challenge.len() == CHALLENGE_LEN && base64url(challenge))
        .ok_or(ParamError::Challenge)?;
    let state = query
        .state
        .filter(|state| (STATE_MIN..=STATE_MAX).contains(&state.len()))
        .filter(|state| state.bytes().all(unreserved))
        .ok_or(ParamError::State)?;
    let device_name = device_name(query.device_name.as_deref().unwrap_or_default())
        .map_err(|_refused| ParamError::DeviceName)?;
    Ok(AuthorizeRequest {
        challenge,
        state,
        device_name,
    })
}

pub fn valid_verifier(verifier: &str) -> bool {
    (VERIFIER_MIN..=VERIFIER_MAX).contains(&verifier.len()) && verifier.bytes().all(unreserved)
}

pub fn challenge_of(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

pub fn percent_encode(text: &str) -> String {
    text.bytes()
        .fold(String::with_capacity(text.len()), |mut out, byte| {
            if unreserved(byte) {
                out.push(char::from(byte));
            } else {
                out.push_str(&format!("%{byte:02X}"));
            }
            out
        })
}

pub fn callback(params: &[(&str, &str)]) -> String {
    let query = params
        .iter()
        .map(|(key, value)| format!("{key}={}", percent_encode(value)))
        .collect::<Vec<_>>()
        .join("&");
    format!("{REDIRECT_URI}?{query}")
}
