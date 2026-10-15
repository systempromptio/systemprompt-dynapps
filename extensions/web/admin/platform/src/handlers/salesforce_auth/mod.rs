//! The Salesforce RFC 7523 JWT-bearer plumbing the connector pre-authorization
//! uses (`services::connector_oauth::salesforce`), and the profile's unlink.
//!
//! Not a login: Entra ID is the login. A user's Salesforce Username is
//! recorded administratively (see
//! [`crate::repositories::connectors::salesforce_identity`]).

mod config;
mod tokens;
mod unlink;

pub use config::SalesforceConfig;
pub(crate) use tokens::post_token_request;
pub(crate) use unlink::salesforce_unlink;

/// Errors from the Salesforce token plumbing.
#[derive(Debug, thiserror::Error)]
pub enum SalesforceError {
    #[error("Salesforce HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Salesforce token endpoint returned {status}: {body}")]
    TokenEndpoint {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("system clock before epoch: {0}")]
    Clock(#[from] std::time::SystemTimeError),
    #[error("Salesforce signing key is not a valid RSA private key: {0}")]
    PrivateKey(#[source] jsonwebtoken::errors::Error),
    #[error("assertion signing failed: {0}")]
    Signing(#[source] jsonwebtoken::errors::Error),
}
