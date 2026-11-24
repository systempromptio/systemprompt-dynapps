//! What a broker call answers when it does not return a tool result.
//!
//! Every refusal carries a stable `error_key` the page's runtime switches on;
//! the message is for people. The keys that name a tool call's outcome are
//! the ones Anthropic's artifact runtime uses (`server_not_connected`,
//! `blocked_by_policy`, `tool_error`, `server_unavailable`, `bad_request`);
//! `reauth`, `cross_site`, `not_found` and `internal` are about the viewer's
//! own request. Its `needs_reauth` has no source yet: core's proxy answers a
//! connector grant that needs signing in again as an unavailable server. The
//! same key is the `outcome` of the call's audit row, beside `ok`.

use std::borrow::Cow;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArtifactCallCode {
    Reauth,
    CrossSite,
    NotFound,
    BadRequest,
    ServerNotConnected,
    BlockedByPolicy,
    ToolError,
    ServerUnavailable,
    Internal,
}

impl ArtifactCallCode {
    #[must_use]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Reauth => "reauth",
            Self::CrossSite => "cross_site",
            Self::NotFound => "not_found",
            Self::BadRequest => "bad_request",
            Self::ServerNotConnected => "server_not_connected",
            Self::BlockedByPolicy => "blocked_by_policy",
            Self::ToolError => "tool_error",
            Self::ServerUnavailable => "server_unavailable",
            Self::Internal => "internal",
        }
    }

    const fn status(self) -> StatusCode {
        match self {
            Self::Reauth => StatusCode::UNAUTHORIZED,
            Self::CrossSite | Self::BlockedByPolicy => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::ServerNotConnected => StatusCode::CONFLICT,
            Self::ToolError => StatusCode::UNPROCESSABLE_ENTITY,
            Self::ServerUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ArtifactCallError {
    code: ArtifactCallCode,
    message: Cow<'static, str>,
}

impl ArtifactCallError {
    pub(crate) const fn new(code: ArtifactCallCode, message: &'static str) -> Self {
        Self {
            code,
            message: Cow::Borrowed(message),
        }
    }

    pub(crate) const fn told(code: ArtifactCallCode, message: String) -> Self {
        Self {
            code,
            message: Cow::Owned(message),
        }
    }

    #[must_use]
    pub(crate) const fn code(&self) -> ArtifactCallCode {
        self.code
    }
}

#[derive(Debug, Serialize)]
struct ErrorBody<'a> {
    error: &'a str,
    error_key: &'static str,
}

impl IntoResponse for ArtifactCallError {
    fn into_response(self) -> Response {
        let status = self.code.status();
        if status.is_server_error() {
            tracing::warn!(error_key = self.code.as_str(), message = %self.message, "artifact tool call failed");
        }
        let body = ErrorBody {
            error: &self.message,
            error_key: self.code.as_str(),
        };
        (status, Json(body)).into_response()
    }
}
