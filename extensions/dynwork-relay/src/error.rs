//! The relay's HTTP errors. Each variant decides its status and its stable
//! `error` key; handlers propagate with `?`.

use axum::Json;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::json;

use crate::auth::AuthError;
use crate::store::StoreError;

#[derive(Debug, thiserror::Error)]
pub enum RelayError {
    #[error("authentication failed")]
    Auth(#[from] AuthError),
    #[error("bad request: {0}")]
    BadRequest(&'static str),
    #[error("not found")]
    NotFound,
    #[error("the pairing token was already used")]
    PairingUsed,
    #[error("the pairing token has expired")]
    PairingExpired,
    #[error("the pairing token is unknown")]
    PairingUnknown,
    #[error("the user has reached the device limit")]
    DeviceLimit,
    #[error("the peer id is already paired")]
    PeerTaken,
    #[error("too many open pairings")]
    TooManyPairings,
    #[error("the relay is at its tunnel limit")]
    Overloaded,
    #[error("a desktop is removed by signing it out, not by revoking it")]
    DesktopNotRevocable,
    #[error("relay store failure")]
    Store(#[from] StoreError),
}

impl RelayError {
    const fn status(&self) -> StatusCode {
        match self {
            Self::Auth(error) => error.status(),
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::PairingUsed | Self::PairingExpired | Self::PairingUnknown => StatusCode::GONE,
            Self::DeviceLimit | Self::PeerTaken | Self::DesktopNotRevocable => StatusCode::CONFLICT,
            Self::TooManyPairings => StatusCode::TOO_MANY_REQUESTS,
            Self::Overloaded => StatusCode::SERVICE_UNAVAILABLE,
            Self::Store(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    const fn key(&self) -> &'static str {
        match self {
            Self::Auth(error) => error.key(),
            Self::BadRequest(_) => "bad_request",
            Self::NotFound => "not_found",
            Self::PairingUsed => "pairing_used",
            Self::PairingExpired => "pairing_expired",
            Self::PairingUnknown => "pairing_unknown",
            Self::DeviceLimit => "device_limit",
            Self::PeerTaken => "peer_taken",
            Self::DesktopNotRevocable => "desktop_not_revocable",
            Self::TooManyPairings => "too_many_pairings",
            Self::Overloaded => "overloaded",
            Self::Store(_) => "internal",
        }
    }
}

impl IntoResponse for RelayError {
    fn into_response(self) -> Response {
        let status = self.status();
        if status.is_server_error() && !matches!(self, Self::Overloaded) {
            tracing::error!(error = ?self, "dynwork relay request failed");
        }
        let message = if status.is_server_error() && !matches!(self, Self::Overloaded) {
            "internal error".to_owned()
        } else {
            self.to_string()
        };
        let mut response = (
            status,
            Json(json!({ "error": self.key(), "message": message })),
        )
            .into_response();
        if matches!(self, Self::Overloaded) {
            response
                .headers_mut()
                .insert(header::RETRY_AFTER, HeaderValue::from_static("1"));
        }
        response
    }
}
