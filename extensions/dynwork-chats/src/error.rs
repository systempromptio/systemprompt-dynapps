//! The extension's error. Each variant decides its HTTP status and its stable
//! `key`; the HTTP layer answers `{ "error": { "key", "message" } }` with them
//! (`http::response`). Handlers propagate with `?`.

use axum::extract::rejection::{BytesRejection, PathRejection, QueryRejection};
use axum::http::StatusCode;
use dynwork_relay::auth::AuthError;

use crate::ids::IdError;
use crate::store::{StoreError, ThreadRecord};

#[derive(Debug, thiserror::Error)]
pub enum ChatsError {
    #[error("invalid identifier")]
    Id(#[from] IdError),
    #[error("chat store failure")]
    Store(#[from] StoreError),
    #[error("authentication failed")]
    Auth(#[from] AuthError),
    #[error("the device credential could not be checked")]
    Devices(#[from] dynwork_relay::store::StoreError),
    #[error("the user's roles could not be read")]
    Users(#[from] systemprompt::traits::auth::AuthProviderError),
    #[error("bad request: {0}")]
    BadRequest(&'static str),
    #[error("the body is not valid JSON of the expected shape")]
    Body(#[from] serde_json::Error),
    #[error("the body is too large")]
    TooLarge,
    #[error("not found")]
    NotFound,
    #[error("this change needs `If-Match: <revision>`")]
    PreconditionRequired,
    #[error("the thread changed since that revision")]
    Conflict(Box<ThreadRecord>),
    #[error("the thread id is already in use")]
    IdTaken,
    #[error("a reply is already running in this thread")]
    Busy,
    #[error("the instance's rules do not allow this user that model")]
    ModelNotAllowed,
    #[error("no model service is available on this instance")]
    AiUnavailable,
}

impl ChatsError {
    pub const fn key(&self) -> &'static str {
        match self {
            Self::Id(IdError::Cursor) => "invalid_cursor",
            Self::Id(IdError::Thread | IdError::Message) => "invalid_id",
            Self::Auth(error) => error.key(),
            Self::BadRequest(_) | Self::Body(_) => "bad_request",
            Self::TooLarge => "too_large",
            Self::NotFound => "not_found",
            Self::PreconditionRequired => "precondition_required",
            Self::Conflict(_) => "revision_conflict",
            Self::IdTaken => "id_taken",
            Self::Busy => "busy",
            Self::AiUnavailable => "ai_unavailable",
            Self::ModelNotAllowed => "model_not_allowed",
            Self::Store(_) | Self::Devices(_) | Self::Users(_) => "internal",
        }
    }

    pub const fn status(&self) -> StatusCode {
        match self {
            Self::Id(_) | Self::BadRequest(_) | Self::Body(_) => StatusCode::BAD_REQUEST,
            Self::Auth(error) => error.status(),
            Self::TooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::PreconditionRequired => StatusCode::PRECONDITION_REQUIRED,
            Self::Conflict(_) | Self::IdTaken | Self::Busy => StatusCode::CONFLICT,
            Self::AiUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::ModelNotAllowed => StatusCode::FORBIDDEN,
            Self::Store(_) | Self::Devices(_) | Self::Users(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<QueryRejection> for ChatsError {
    fn from(rejection: QueryRejection) -> Self {
        tracing::debug!(error = %rejection, "chats query string refused");
        Self::BadRequest("the query string is malformed")
    }
}

impl From<PathRejection> for ChatsError {
    fn from(rejection: PathRejection) -> Self {
        tracing::debug!(error = %rejection, "chats path refused");
        Self::NotFound
    }
}

impl From<BytesRejection> for ChatsError {
    fn from(rejection: BytesRejection) -> Self {
        if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE {
            Self::TooLarge
        } else {
            tracing::debug!(error = %rejection, "chats body refused");
            Self::BadRequest("the body could not be read")
        }
    }
}
