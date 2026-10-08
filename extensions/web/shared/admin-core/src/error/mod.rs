//! The admin plane's HTTP error type.
//!
//! Domain errors convert in via `From`, and the variant alone decides the
//! status code, so handlers propagate with a bare `?` rather than mapping at
//! each call site. Logging happens once, in `into_response`, at a severity
//! set by the status class, for both the JSON and the HTML face.
//!
//! `public_message` is the only text that reaches the wire. Typed validation
//! causes (`Invalid`, `Archive`) surface their own words as the detail;
//! credential, upstream and internal causes are logged in full but never sent.
//! `NotImplemented` is a mounted route whose handler has not landed, kept
//! apart from `NotFound` so the client is not told the endpoint is wrong.

mod from;
mod html;
mod managed;
mod orchestration;

pub use html::{AdminHtmlError, AdminHtmlResult};

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::crypto::secret_crypto::SecretCryptoError;
use crate::repositories::bridge::BridgeRepoError;
use crate::request::ErrorBody;
use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, Error)]
pub enum AdminError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unprocessable request: {0}")]
    Unprocessable(String),

    #[error("Archive unreadable: {0}")]
    Archive(#[from] zip::result::ZipError),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Authentication failed: {0}")]
    Unauthenticated(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Too many requests: {0}")]
    RateLimited(String),

    #[error("Unavailable: {0}")]
    Unavailable(String),

    #[error("Upstream error: {0}")]
    Upstream(String),

    #[error("Upstream error: {context}: {source}")]
    UpstreamFailure {
        context: &'static str,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Invalid input: {context}: {source}")]
    Invalid {
        context: &'static str,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Forbidden: {0}")]
    Refused(#[source] crate::types::role::RoleChangeRefusal),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Access token repository error: {0}")]
    BridgeRepo(BridgeRepoError),

    #[error("Marketplace error: {0}")]
    Marketplace(MarketplaceError),

    #[error("Crypto error: {0}")]
    Crypto(#[from] SecretCryptoError),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Internal error: {0}")]
    Internal(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl AdminError {
    #[must_use]
    pub fn internal<E>(err: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self::Internal(err.into())
    }

    #[must_use]
    pub fn upstream<E>(context: &'static str, source: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self::UpstreamFailure {
            context,
            source: source.into(),
        }
    }

    #[must_use]
    pub fn invalid<E>(context: &'static str, source: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self::Invalid {
            context,
            source: source.into(),
        }
    }

    #[must_use]
    pub const fn status(&self) -> StatusCode {
        match self {
            Self::NotFound(_) | Self::Marketplace(MarketplaceError::NotFound(_)) => {
                StatusCode::NOT_FOUND
            },
            Self::BadRequest(_)
            | Self::Invalid { .. }
            | Self::BridgeRepo(BridgeRepoError::Validation(_))
            | Self::Marketplace(MarketplaceError::BadRequest(_)) => StatusCode::BAD_REQUEST,
            Self::Unprocessable(_) | Self::Archive(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Unauthorized(_) | Self::Unauthenticated(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) | Self::Refused(_) => StatusCode::FORBIDDEN,
            Self::Conflict(_) | Self::Marketplace(MarketplaceError::Conflict(_)) => {
                StatusCode::CONFLICT
            },
            Self::RateLimited(_) => StatusCode::TOO_MANY_REQUESTS,
            Self::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Upstream(_) | Self::UpstreamFailure { .. } => StatusCode::BAD_GATEWAY,
            Self::NotImplemented(_) => StatusCode::NOT_IMPLEMENTED,
            Self::Database(_)
            | Self::BridgeRepo(_)
            | Self::Marketplace(_)
            | Self::Crypto(_)
            | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub(super) fn public_message(&self) -> String {
        match self {
            Self::NotFound(msg)
            | Self::Unprocessable(msg)
            | Self::BadRequest(msg)
            | Self::Unauthorized(msg)
            | Self::Forbidden(msg)
            | Self::Conflict(msg)
            | Self::RateLimited(msg)
            | Self::Unavailable(msg)
            | Self::NotImplemented(msg)
            | Self::BridgeRepo(BridgeRepoError::Validation(msg))
            | Self::Marketplace(
                MarketplaceError::BadRequest(msg)
                | MarketplaceError::NotFound(msg)
                | MarketplaceError::Conflict(msg),
            ) => msg.clone(),
            Self::Invalid { context, source } => format!("{context}: {source}"),
            Self::Archive(source) => format!("archive unreadable: {source}"),
            Self::Refused(refusal) => refusal.to_string(),
            Self::Upstream(_) | Self::UpstreamFailure { .. } => "Upstream service error".to_owned(),
            Self::Unauthenticated(_) => "Unauthorized".to_owned(),
            Self::Crypto(_) => "Internal configuration error".to_owned(),
            Self::Database(_) | Self::BridgeRepo(_) | Self::Marketplace(_) | Self::Internal(_) => {
                "Internal server error".to_owned()
            },
        }
    }

    #[must_use]
    pub fn unauthenticated<E>(err: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self::Unauthenticated(err.into())
    }

    pub(super) fn log(&self, status: StatusCode) {
        if status.is_server_error() {
            tracing::error!(error = %self, "Admin handler returned server error");
        } else {
            tracing::warn!(error = %self, "Admin handler returned client error");
        }
    }
}

impl IntoResponse for AdminError {
    fn into_response(self) -> Response {
        let status = self.status();
        self.log(status);
        let body = Json(ErrorBody {
            error: self.public_message(),
        });
        (status, body).into_response()
    }
}

pub type AdminResult<T> = Result<T, AdminError>;
