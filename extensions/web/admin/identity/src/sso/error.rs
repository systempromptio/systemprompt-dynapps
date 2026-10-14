//! What an Entra ID provider can fail on, with the typed cause kept.
//!
//! Core's [`SsoError`] carries a reason string, so the cause is rendered once,
//! in the [`From`] impl, and each variant keeps the `SsoError` kind — and so
//! the reason code — the sign-in refusal reports.

use systemprompt::sso::SsoError;

#[derive(Debug, thiserror::Error)]
pub enum SsoProviderError {
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
    #[error("SAMLResponse is not base64: {0}")]
    SamlBase64(#[source] base64::DecodeError),
    #[error("IdP metadata unusable: {0}")]
    IdpMetadata(#[source] saml::Error),
    #[error("IdP federation metadata client: {0}")]
    MetadataClient(#[source] reqwest::Error),
    #[error("IdP federation metadata fetch from {url} failed: {source}")]
    MetadataFetch {
        url: String,
        #[source]
        source: reqwest::Error,
    },
    #[error("IdP federation metadata from {url} does not parse: {source}")]
    MetadataParse {
        url: String,
        #[source]
        source: saml::Error,
    },
    #[error("service-provider configuration rejected: {0}")]
    SpConfig(#[source] saml::Error),
    #[error("AuthnRequest could not be built: {0}")]
    AuthnRequest(#[source] saml::Error),
    #[error("SP metadata: {0}")]
    SpMetadata(#[source] saml::Error),
}

impl From<SsoProviderError> for SsoError {
    fn from(err: SsoProviderError) -> Self {
        match err {
            SsoProviderError::TokenParse(_)
            | SsoProviderError::IdTokenHeader(_)
            | SsoProviderError::IdTokenRejected(_)
            | SsoProviderError::IdTokenClaims(_)
            | SsoProviderError::SamlBase64(_) => Self::InvalidAssertion(err.to_string()),
            SsoProviderError::Discovery { .. }
            | SsoProviderError::DiscoveryStatus { .. }
            | SsoProviderError::DiscoveryParse { .. }
            | SsoProviderError::TokenEndpoint(_)
            | SsoProviderError::IdpMetadata(_)
            | SsoProviderError::MetadataClient(_)
            | SsoProviderError::MetadataFetch { .. }
            | SsoProviderError::MetadataParse { .. }
            | SsoProviderError::SpConfig(_)
            | SsoProviderError::AuthnRequest(_)
            | SsoProviderError::SpMetadata(_) => Self::Unavailable(err.to_string()),
        }
    }
}
