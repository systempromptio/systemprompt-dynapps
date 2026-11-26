//! How a failure answers over HTTP.
//!
//! The body is `{ "error": { "key", "message" } }`, with
//! a stable `key` and the status that goes with it. A server error is logged
//! and answered with a fixed message, so nothing internal reaches the client.
//! No answer is cached.

use axum::Json;
use axum::extract::rejection::{BytesRejection, PathRejection, QueryRejection};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use dynwork_relay::auth::AuthError;
use dynwork_relay::store::StoreError;
use serde::Serialize;

use crate::brain::fault::BrainFault;

#[derive(Debug, thiserror::Error)]
pub enum HttpFault {
    #[error("authentication failed")]
    Auth(#[from] AuthError),
    #[error("the device credential could not be checked")]
    Devices(#[from] StoreError),
    #[error(transparent)]
    Brain(#[from] BrainFault),
    #[error("bad request: {0}")]
    BadRequest(&'static str),
    #[error("the body is not valid JSON of the expected shape")]
    Body(#[from] serde_json::Error),
    #[error("not found")]
    NotFound,
}

impl HttpFault {
    pub const fn key(&self) -> &'static str {
        match self {
            Self::Auth(error) => error.key(),
            Self::Devices(_) | Self::Brain(BrainFault::Internal(_)) => "internal",
            Self::BadRequest(_) | Self::Body(_) => "bad_request",
            Self::NotFound | Self::Brain(BrainFault::NotFound) => "not_found",
            Self::Brain(BrainFault::Forbidden) => "forbidden",
            Self::Brain(
                BrainFault::Invalid(_) | BrainFault::Refused(_) | BrainFault::Property(_),
            ) => "invalid",
            Self::Brain(BrainFault::Unavailable(_)) => "unavailable",
        }
    }

    pub const fn status(&self) -> StatusCode {
        match self {
            Self::Auth(error) => error.status(),
            Self::Devices(_) | Self::Brain(BrainFault::Internal(_)) => {
                StatusCode::INTERNAL_SERVER_ERROR
            },
            Self::BadRequest(_)
            | Self::Body(_)
            | Self::Brain(
                BrainFault::Invalid(_) | BrainFault::Refused(_) | BrainFault::Property(_),
            ) => StatusCode::BAD_REQUEST,
            Self::NotFound | Self::Brain(BrainFault::NotFound) => StatusCode::NOT_FOUND,
            Self::Brain(BrainFault::Forbidden) => StatusCode::FORBIDDEN,
            Self::Brain(BrainFault::Unavailable(_)) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

impl From<PathRejection> for HttpFault {
    fn from(rejection: PathRejection) -> Self {
        tracing::debug!(error = %rejection, "dynbrain path refused");
        Self::NotFound
    }
}

impl From<QueryRejection> for HttpFault {
    fn from(rejection: QueryRejection) -> Self {
        tracing::debug!(error = %rejection, "dynbrain query string refused");
        Self::BadRequest("the query string is malformed")
    }
}

impl From<BytesRejection> for HttpFault {
    fn from(rejection: BytesRejection) -> Self {
        tracing::debug!(error = %rejection, "dynbrain body refused");
        Self::BadRequest("the body could not be read")
    }
}

#[derive(Debug, Serialize)]
struct ErrorDetail {
    key: &'static str,
    message: String,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: ErrorDetail,
}

impl IntoResponse for HttpFault {
    fn into_response(self) -> Response {
        let status = self.status();
        let message = if status.is_server_error() && status != StatusCode::SERVICE_UNAVAILABLE {
            tracing::error!(error = ?self, "dynbrain request failed");
            "internal error".to_owned()
        } else {
            self.to_string()
        };
        let body = ErrorBody {
            error: ErrorDetail {
                key: self.key(),
                message,
            },
        };
        let mut response = (status, Json(body)).into_response();
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
        response
    }
}
