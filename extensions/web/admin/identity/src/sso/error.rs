//! What an Entra ID provider can fail on, with the typed cause kept.
//!
//! Core's [`SsoError`] carries a reason string, so the cause is rendered once,
//! in the [`From`] impl, and each variant keeps the `SsoError` kind — and so
//! the reason code — the sign-in refusal reports.

use systemprompt::sso::SsoError;

#[derive(Debug, thiserror::Error)]
pub enum SsoProviderError {
    #[error("outbound destination {url} refused: {source}")]
    OutboundUrl {
        url: String,
        #[source]
        source: systemprompt::models::net::OutboundUrlError,
    },
    #[error("JWKS uri {url} is not on an allowed host")]
    JwksHost { url: String },
    #[error("discovery request to {url} failed: {source}")]
    Discovery {
        url: String,
        #[source]
        source: reqwest::Error,
    },
    #[error("discovery {url} answered {status}")]
    DiscoveryStatus {
        url: String,
        status: reqwest::StatusCode,
    },
    #[error("discovery {url} unparseable: {source}")]
    DiscoveryParse {
        url: String,
        #[source]
        source: reqwest::Error,
    },
    #[error("JWKS fetch from {url} failed: {source}")]
    JwksFetch {
        url: String,
        #[source]
        source: reqwest::Error,
    },
    #[error("JWKS from {url} unparseable: {source}")]
    JwksParse {
        url: String,
        #[source]
        source: reqwest::Error,
    },
    #[error("token endpoint unreachable: {0}")]
    TokenEndpoint(#[source] reqwest::Error),
    #[error("token response unparseable: {0}")]
    TokenParse(#[source] reqwest::Error),
    #[error("id_token header: {0}")]
    IdTokenHeader(#[source] jsonwebtoken::errors::Error),
    #[error("id_token rejected: {0}")]
    IdTokenRejected(#[source] jsonwebtoken::errors::Error),
    #[error("id_token claims: {0}")]
    IdTokenClaims(#[source] serde_json::Error),
}

/// Why an enabled Entra ID provider cannot be built at boot.
#[derive(Debug, thiserror::Error)]
pub enum EntraSetupError {
    #[error("tenant_id, client_id or redirect_uri is unset")]
    Incomplete,
    #[error("client secret `{key}` is missing from the secrets store")]
    ClientSecretMissing { key: String },
    #[error("provider could not be built: {0}")]
    Build(#[source] SsoError),
}

impl From<SsoProviderError> for SsoError {
    fn from(err: SsoProviderError) -> Self {
        match err {
            SsoProviderError::TokenParse(_)
            | SsoProviderError::IdTokenHeader(_)
            | SsoProviderError::IdTokenRejected(_)
            | SsoProviderError::IdTokenClaims(_) => Self::InvalidAssertion(err.to_string()),
            SsoProviderError::OutboundUrl { .. }
            | SsoProviderError::JwksHost { .. }
            | SsoProviderError::Discovery { .. }
            | SsoProviderError::DiscoveryStatus { .. }
            | SsoProviderError::DiscoveryParse { .. }
            | SsoProviderError::TokenEndpoint(_)
            | SsoProviderError::JwksFetch { .. }
            | SsoProviderError::JwksParse { .. } => Self::Unavailable(err.to_string()),
        }
    }
}
