//! The desktop API's errors. Each variant decides its status and its stable
//! `error` key; handlers propagate with `?`.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use dynwork_relay::auth::AuthError;
use serde_json::json;

use crate::store::StoreError;
use crate::view::RoutineView;

#[derive(Debug, thiserror::Error)]
pub enum RoutinesError {
    #[error("authentication failed")]
    Auth(#[from] AuthError),
    #[error("bad request: {0}")]
    BadRequest(&'static str),
    #[error("not found")]
    NotFound,
    #[error("the routine changed since the revision named in If-Match")]
    Conflict(Box<RoutineView>),
    #[error("only the routine's owner device reports its runs")]
    NotOwnerDevice,
    #[error("the run id belongs to another routine")]
    RunConflict,
    #[error("routines store failure")]
    Store(#[from] StoreError),
}

impl RoutinesError {
    const fn status(&self) -> StatusCode {
        match self {
            Self::Auth(error) => error.status(),
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict(_) | Self::RunConflict => StatusCode::CONFLICT,
            Self::NotOwnerDevice => StatusCode::FORBIDDEN,
            Self::Store(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    const fn key(&self) -> &'static str {
        match self {
            Self::Auth(error) => error.key(),
            Self::BadRequest(_) => "bad_request",
            Self::NotFound => "not_found",
            Self::Conflict(_) => "revision_conflict",
            Self::NotOwnerDevice => "not_owner_device",
            Self::RunConflict => "run_conflict",
            Self::Store(_) => "internal",
        }
    }
}

impl IntoResponse for RoutinesError {
    fn into_response(self) -> Response {
        let status = self.status();
        if status.is_server_error() {
            tracing::error!(error = ?self, "dynwork routines request failed");
        }
        let message = if status.is_server_error() {
            "internal error".to_owned()
        } else {
            self.to_string()
        };
        // JSON: outgoing fixed-shape construction (the error body).
        let body = match &self {
            Self::Conflict(current) => {
                json!({ "error": self.key(), "message": message, "current": current })
            },
            _ => json!({ "error": self.key(), "message": message }),
        };
        (status, Json(body)).into_response()
    }
}
