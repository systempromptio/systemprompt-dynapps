//! The desktop API's errors. Each variant decides its status and its stable
//! `error` key; handlers propagate with `?`.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use dynwork_relay::auth::AuthError;
use serde_json::json;
use systemprompt::governance::authz::AuthzError;
use systemprompt::models::authz::SubjectSourceError;

use crate::store::StoreError;
use crate::template::BadLink;
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
    #[error("a live routine of this template already exists")]
    AlreadyActivated(Box<RoutineView>),
    #[error("a field the template locks differs from the revision the routine names")]
    TemplateLocked(Option<Box<RoutineView>>),
    #[error("the routine follows a template that is mandatory for you; pause it instead")]
    TemplateMandatory,
    #[error(transparent)]
    BadLink(#[from] BadLink),
    #[error("routines store failure")]
    Store(#[from] StoreError),
    #[error("access rules could not be resolved")]
    Access(#[from] AuthzError),
    #[error("subject attributes could not be resolved")]
    Subjects(#[from] SubjectSourceError),
    #[error("the answer could not be encoded")]
    Encode(#[from] serde_json::Error),
}

impl RoutinesError {
    const fn status(&self) -> StatusCode {
        match self {
            Self::Auth(error) => error.status(),
            Self::BadRequest(_) | Self::BadLink(_) => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict(_)
            | Self::RunConflict
            | Self::AlreadyActivated(_)
            | Self::TemplateLocked(_)
            | Self::TemplateMandatory => StatusCode::CONFLICT,
            Self::NotOwnerDevice => StatusCode::FORBIDDEN,
            Self::Store(_) | Self::Access(_) | Self::Subjects(_) | Self::Encode(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            },
        }
    }

    const fn key(&self) -> &'static str {
        match self {
            Self::Auth(error) => error.key(),
            Self::BadRequest(_) | Self::BadLink(_) => "bad_request",
            Self::NotFound => "not_found",
            Self::Conflict(_) => "revision_conflict",
            Self::NotOwnerDevice => "not_owner_device",
            Self::RunConflict => "run_conflict",
            Self::AlreadyActivated(_) => "already_activated",
            Self::TemplateLocked(_) => "template_locked",
            Self::TemplateMandatory => "template_mandatory",
            Self::Store(_) | Self::Access(_) | Self::Subjects(_) | Self::Encode(_) => "internal",
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
            Self::TemplateLocked(current) => {
                json!({ "error": self.key(), "message": message, "current": current })
            },
            Self::AlreadyActivated(routine) => {
                json!({ "error": self.key(), "message": message, "routine": routine })
            },
            _ => json!({ "error": self.key(), "message": message }),
        };
        (status, Json(body)).into_response()
    }
}
