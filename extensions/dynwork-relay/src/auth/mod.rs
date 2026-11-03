//! Who is on a socket or a request.
//!
//! Desktops present the `DynWork` JWT (audience `bridge` or `api`), verified by
//! the instance's token authority. Phones present a device credential,
//! checked by the chain of [`DeviceVerifier`]s the relay was built with: the
//! first verifier that recognises the token decides. The relay ships one,
//! [`device::CredentialVerifier`], for the credentials a pairing claim
//! issues; another sign-in path adds its own verifier to the chain without
//! touching the socket code.

pub mod desktop;
pub mod device;

use axum::http::{HeaderMap, StatusCode, header};
use systemprompt::models::auth::TokenError;

pub use desktop::{DESKTOP_AUDIENCES, DesktopIdentity, verify_desktop};
pub use device::{CredentialVerifier, DeviceVerifier, VerifiedDevice};

use crate::store::StoreError;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("no bearer token")]
    Missing,
    #[error("the bearer token is malformed")]
    Malformed,
    #[error("the token was rejected")]
    Rejected(#[source] TokenError),
    #[error("unknown or revoked device credential")]
    UnknownDevice,
    #[error("the account no longer exists")]
    UnknownUser,
    #[error("the token does not belong to a signed-in person")]
    NotAPerson,
    #[error("the token has expired")]
    Expired,
    #[error("the credential could not be checked")]
    Store(#[from] StoreError),
}

impl AuthError {
    pub const fn status(&self) -> StatusCode {
        match self {
            Self::Store(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::UNAUTHORIZED,
        }
    }

    pub const fn key(&self) -> &'static str {
        match self {
            Self::Store(_) => "internal",
            _ => "unauthorized",
        }
    }
}

pub fn bearer(headers: &HeaderMap) -> Result<&str, AuthError> {
    let value = headers
        .get(header::AUTHORIZATION)
        .ok_or(AuthError::Missing)?
        .to_str()
        .map_err(|source| {
            tracing::debug!(error = %source, "authorization header is not ASCII");
            AuthError::Malformed
        })?;
    let token = value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))
        .ok_or(AuthError::Malformed)?
        .trim();
    if token.is_empty() {
        return Err(AuthError::Missing);
    }
    Ok(token)
}
