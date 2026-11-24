//! What core's MCP proxy answered, in the broker's codes.
//!
//! The proxy's HTTP status says whether it let the viewer through; a JSON or
//! SSE body carries the JSON-RPC response, the message whose `id` is the
//! request's; and a JSON-RPC error says whether the server refused the viewer,
//! the request was wrong, or the tool itself failed.

use axum::http::StatusCode;
// JSON: MCP-protocol boundary, JSON-RPC messages carry schema-less payloads.
use serde_json::Value;

use super::error::{ArtifactCallCode, ArtifactCallError};

const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;

pub(super) fn refuse_status(status: StatusCode) -> Result<(), ArtifactCallError> {
    if status.is_success() {
        return Ok(());
    }
    let (code, message) = match status {
        StatusCode::UNAUTHORIZED => (
            ArtifactCallCode::Reauth,
            "the viewer's session was refused; sign in again",
        ),
        StatusCode::FORBIDDEN => (
            ArtifactCallCode::BlockedByPolicy,
            "the viewer may not use this server",
        ),
        StatusCode::CONFLICT => (
            ArtifactCallCode::ServerNotConnected,
            "the viewer has not connected this server",
        ),
        StatusCode::BAD_REQUEST | StatusCode::PAYLOAD_TOO_LARGE => (
            ArtifactCallCode::BadRequest,
            "the server refused the request",
        ),
        _ => (
            ArtifactCallCode::ServerUnavailable,
            "the server is unavailable",
        ),
    };
    tracing::debug!(%status, error_key = code.as_str(), "the MCP proxy refused an artifact tool call");
    Err(ArtifactCallError::new(code, message))
}

// JSON: MCP-protocol boundary, the JSON-RPC message answering `id`.
pub(super) fn find_reply(body: &[u8], sse: bool, id: u64) -> Option<Value> {
    let answers = |message: &Value| message.get("id").and_then(Value::as_u64) == Some(id);
    if !sse {
        return serde_json::from_slice::<Value>(body).ok().filter(answers);
    }
    let text = std::str::from_utf8(body).ok()?;
    text.replace("\r\n", "\n")
        .split("\n\n")
        .filter_map(|event| {
            let data: Vec<&str> = event
                .lines()
                .filter_map(|line| line.strip_prefix("data:"))
                .map(|data| data.strip_prefix(' ').unwrap_or(data))
                .collect();
            serde_json::from_str::<Value>(&data.join("\n")).ok()
        })
        .find(answers)
}

// JSON: MCP-protocol boundary, a JSON-RPC result or error object.
pub(super) fn rpc_result(mut reply: Value) -> Result<Value, ArtifactCallError> {
    if let Some(result) = reply.get_mut("result") {
        return Ok(result.take());
    }
    let error = reply.get("error").ok_or_else(unreadable)?;
    let message = error
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("the tool failed")
        .to_owned();
    let forbidden = error
        .pointer("/data/reason")
        .and_then(Value::as_str)
        .is_some_and(|reason| reason == "forbidden");
    let code = match error.get("code").and_then(Value::as_i64) {
        _ if forbidden => ArtifactCallCode::BlockedByPolicy,
        Some(code) => rpc_code(code),
        None => ArtifactCallCode::ToolError,
    };
    Err(ArtifactCallError::told(code, message))
}

// Why: core's MCP host refuses a caller with `invalid_request` (no verified
// hop, the server's access rules, a tool the caller may not see) and an
// unknown tool or bad arguments with `invalid_params`; any other error is the
// tool's own.
const fn rpc_code(code: i64) -> ArtifactCallCode {
    match code {
        INVALID_REQUEST => ArtifactCallCode::BlockedByPolicy,
        METHOD_NOT_FOUND | INVALID_PARAMS => ArtifactCallCode::BadRequest,
        _ => ArtifactCallCode::ToolError,
    }
}

pub(super) const fn unavailable() -> ArtifactCallError {
    ArtifactCallError::new(
        ArtifactCallCode::ServerUnavailable,
        "the server is unavailable",
    )
}

pub(super) const fn unreadable() -> ArtifactCallError {
    ArtifactCallError::new(
        ArtifactCallCode::ServerUnavailable,
        "the server's answer could not be read",
    )
}
