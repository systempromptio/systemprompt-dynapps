//! The portal's streamable-HTTP MCP endpoints: JSON responses (no SSE), one
//! `Mcp-Session-Id` per `initialize`, and a bearer check on every request.

use axum::http::{HeaderMap, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

use super::Inner;
use super::routes::{error, json_ok, unauthorized};

const SESSION_HEADER: &str = "mcp-session-id";

pub(super) fn handle(
    inner: &Inner,
    method_and_path: (&Method, &str),
    headers: &HeaderMap,
    authorization: Option<&str>,
    body: &str,
) -> Response {
    let (method, path) = method_and_path;
    let Some(subject) = inner.subject_of_header(authorization) else {
        return unauthorized(inner, path);
    };
    let session = headers
        .get(SESSION_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    if *method == Method::DELETE {
        if let Some(id) = session {
            inner.sessions.lock().expect("sessions").remove(&id);
        }
        return StatusCode::OK.into_response();
    }
    if *method != Method::POST {
        return error(StatusCode::METHOD_NOT_ALLOWED, "POST or DELETE");
    }
    let Ok(request) = serde_json::from_str::<Value>(body) else {
        return error(StatusCode::BAD_REQUEST, "invalid JSON-RPC");
    };
    let rpc_method = request["method"].as_str().unwrap_or_default();
    if rpc_method == "initialize" {
        let id = inner.next("portal-session");
        inner.sessions.lock().expect("sessions").insert(id.clone());
        let mut response = json_ok(&json!({
            "jsonrpc": "2.0", "id": request["id"],
            "result": {
                "protocolVersion": request["params"]["protocolVersion"].as_str().unwrap_or("2025-03-26"),
                "capabilities": { "tools": {} },
                "serverInfo": { "name": server_name(path), "version": "1.0.0" },
            },
        }));
        response.headers_mut().insert(
            SESSION_HEADER,
            HeaderValue::from_str(&id).expect("session id is ASCII"),
        );
        return response;
    }
    let known = session
        .as_ref()
        .is_some_and(|id| inner.sessions.lock().expect("sessions").contains(id));
    if !known {
        return error(StatusCode::NOT_FOUND, "unknown or missing Mcp-Session-Id");
    }
    if request.get("id").is_none() {
        return StatusCode::ACCEPTED.into_response();
    }
    let result = match rpc_method {
        "tools/list" => json!({ "tools": tools() }),
        "tools/call" => call(&request["params"], &subject, path),
        _ => {
            return json_ok(&json!({
                "jsonrpc": "2.0", "id": request["id"],
                "error": { "code": -32601, "message": "method not found" },
            }));
        },
    };
    json_ok(&json!({ "jsonrpc": "2.0", "id": request["id"], "result": result }))
}

fn server_name(path: &str) -> &'static str {
    if path == super::DYNBRAIN_PATH {
        "dynbrain"
    } else {
        "dynapps-odoo"
    }
}

fn tools() -> Value {
    json!([
        {
            "name": "whoami",
            "description": "The portal account this bearer belongs to.",
            "inputSchema": { "type": "object", "properties": {} },
        },
        {
            "name": "echo",
            "description": "Echo `text` back.",
            "inputSchema": {
                "type": "object",
                "properties": { "text": { "type": "string" } },
                "required": ["text"],
            },
        },
    ])
}

fn call(params: &Value, subject: &str, path: &str) -> Value {
    let text = match params["name"].as_str() {
        Some("whoami") => format!("{subject}@{}", server_name(path)),
        Some("echo") => params["arguments"]["text"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
        _ => {
            return json!({
                "isError": true,
                "content": [{ "type": "text", "text": "unknown tool" }],
            });
        },
    };
    json!({ "content": [{ "type": "text", "text": text }], "isError": false })
}
