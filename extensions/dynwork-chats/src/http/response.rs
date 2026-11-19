//! How a [`ChatsError`] and a thread answer over HTTP.
//!
//! Errors are `{ "error": { "key", "message" } }`; a revision conflict adds
//! `current`, the thread as it is now. A server error is logged and answered
//! with a fixed message, so nothing internal reaches the client. A thread
//! answers with `ETag: "<revision>"`. No answer is cached.

use axum::Json;
use axum::http::{HeaderValue, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::error::ChatsError;
use crate::http::wire::WireThread;
use crate::store::ThreadRecord;

#[derive(Debug, Serialize)]
struct ErrorDetail {
    key: &'static str,
    message: String,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: ErrorDetail,
    #[serde(skip_serializing_if = "Option::is_none")]
    current: Option<WireThread>,
}

impl IntoResponse for ChatsError {
    fn into_response(self) -> Response {
        let status = self.status();
        let message = if status.is_server_error() {
            tracing::error!(error = ?self, "dynwork chats request failed");
            "internal error".to_owned()
        } else {
            self.to_string()
        };
        let current = match &self {
            Self::Conflict(record) => Some(WireThread::from(record.as_ref())),
            _ => None,
        };
        let body = ErrorBody {
            error: ErrorDetail {
                key: self.key(),
                message,
            },
            current,
        };
        no_store((status, Json(body)).into_response())
    }
}

pub fn thread(record: &ThreadRecord) -> Response {
    let mut response = Json(WireThread::from(record)).into_response();
    if let Ok(tag) = HeaderValue::from_str(&format!("\"{}\"", record.revision)) {
        response.headers_mut().insert(header::ETAG, tag);
    }
    no_store(response)
}

pub fn no_store(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}
