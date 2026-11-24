//! What a broker request answers when it does not succeed.
//!
//! Every refusal carries a stable `error_key` the page's runtime switches on;
//! the message is for people. The keys that name a tool call's or a
//! publish's outcome are the ones Anthropic's artifact runtime uses
//! (`server_not_connected`, `not_in_manifest`, `blocked_by_policy`,
//! `approval_required`, `tool_error`, `server_unavailable`, `bad_request`;
//! `conflict`, `not_writer`, `not_declared`, `too_large`, `invalid_content`);
//! `reauth`, `cross_site`, `not_found` and `internal` are about the viewer's
//! own request. Its `needs_reauth` has no source yet: core's proxy answers a
//! connector grant that needs signing in again as an unavailable server. The
//! same key is the `outcome` of a call's audit row, beside `ok` and `cached`.
//! A `conflict` names the version that is live (`live`).

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
    NotInManifest,
    BlockedByPolicy,
    ApprovalRequired,
    ToolError,
    ServerUnavailable,
    Conflict,
    NotWriter,
    NotDeclared,
    TooLarge,
    InvalidContent,
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
            Self::NotInManifest => "not_in_manifest",
            Self::BlockedByPolicy => "blocked_by_policy",
            Self::ApprovalRequired => "approval_required",
            Self::ToolError => "tool_error",
            Self::ServerUnavailable => "server_unavailable",
            Self::Conflict => "conflict",
            Self::NotWriter => "not_writer",
            Self::NotDeclared => "not_declared",
            Self::TooLarge => "too_large",
            Self::InvalidContent => "invalid_content",
            Self::Internal => "internal",
        }
    }

    const fn status(self) -> StatusCode {
        match self {
            Self::Reauth => StatusCode::UNAUTHORIZED,
            Self::CrossSite
            | Self::NotInManifest
            | Self::BlockedByPolicy
            | Self::NotWriter
            | Self::NotDeclared => StatusCode::FORBIDDEN,
            Self::ApprovalRequired => StatusCode::PRECONDITION_REQUIRED,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::BadRequest | Self::InvalidContent => StatusCode::BAD_REQUEST,
            Self::ServerNotConnected | Self::Conflict => StatusCode::CONFLICT,
            Self::TooLarge => StatusCode::PAYLOAD_TOO_LARGE,
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
    live: Option<String>,
}

impl ArtifactCallError {
    pub(crate) const fn new(code: ArtifactCallCode, message: &'static str) -> Self {
        Self {
            code,
            message: Cow::Borrowed(message),
            live: None,
        }
    }

    pub(crate) const fn told(code: ArtifactCallCode, message: String) -> Self {
        Self {
            code,
            message: Cow::Owned(message),
            live: None,
        }
    }

    pub(crate) const fn conflict(live: String) -> Self {
        Self {
            code: ArtifactCallCode::Conflict,
            message: Cow::Borrowed("a newer version of this page was published first"),
            live: Some(live),
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
    #[serde(skip_serializing_if = "Option::is_none")]
    live: Option<&'a str>,
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
            live: self.live.as_deref(),
        };
        (status, Json(body)).into_response()
    }
}
