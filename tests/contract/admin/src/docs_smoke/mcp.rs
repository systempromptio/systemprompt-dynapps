//! A minimal MCP client over the streamable HTTP transport: initialize, then
//! JSON-RPC requests carrying the session id, the caller's bearer and the
//! request-context headers the server builds the caller's context from. A
//! response arrives as JSON or as one server-sent event; both are read.

use serde_json::{Value, json};
use systemprompt::identifiers::{ContextId, headers};

const PROTOCOL_VERSION: &str = "2025-06-18";
const MCP_SESSION: &str = "mcp-session-id";

pub(super) struct McpClient {
    http: reqwest::Client,
    url: String,
    bearer: String,
    user_id: String,
    context_id: String,
    session: String,
    next_id: u64,
}

pub(super) struct ToolResult {
    pub(super) raw: Value,
    pub(super) is_error: bool,
    pub(super) payload: Value,
}

impl ToolResult {
    // What a client hands the model: the content, the structured content and
    // the error flag. `_meta` carries the execution envelope for the host and
    // is left out.
    pub(super) fn model_bytes(&self) -> usize {
        let mut visible = self.raw.clone();
        if let Some(object) = visible.as_object_mut() {
            object.remove("_meta");
        }
        serde_json::to_string(&visible)
            .expect("a result serialises")
            .len()
    }

    pub(super) fn wire_bytes(&self) -> usize {
        serde_json::to_string(&self.raw)
            .expect("a result serialises")
            .len()
    }
}

fn read_body(content_type: &str, body: &str) -> Value {
    let json = if content_type.starts_with("text/event-stream") {
        body.lines()
            .filter_map(|line| line.strip_prefix("data:"))
            .map(str::trim)
            .rfind(|data| !data.is_empty())
            .unwrap_or_else(|| panic!("no event data in {body:?}"))
            .to_owned()
    } else {
        body.to_owned()
    };
    serde_json::from_str(&json).unwrap_or_else(|e| panic!("MCP response {json:?}: {e}"))
}

fn payload(result: &Value) -> Value {
    if let Some(structured) = result.get("structuredContent") {
        return structured.clone();
    }
    let text = result["content"][0]["text"].as_str().unwrap_or_default();
    serde_json::from_str(text).unwrap_or_else(|_| Value::String(text.to_owned()))
}

impl McpClient {
    pub(super) async fn connect(url: &str, bearer: &str, user_id: &str) -> Self {
        let mut client = Self {
            http: reqwest::Client::new(),
            url: url.to_owned(),
            bearer: bearer.to_owned(),
            user_id: user_id.to_owned(),
            context_id: ContextId::generate().as_str().to_owned(),
            session: String::new(),
            next_id: 0,
        };
        let response = client
            .post(&json!({
                "jsonrpc": "2.0",
                "id": 0,
                "method": "initialize",
                "params": {
                    "protocolVersion": PROTOCOL_VERSION,
                    "capabilities": {},
                    "clientInfo": {"name": "docs-smoke", "version": "1"},
                },
            }))
            .await;
        client.session = response
            .headers()
            .get(MCP_SESSION)
            .and_then(|value| value.to_str().ok())
            .expect("initialize returns a session id")
            .to_owned();
        let status = response.status();
        assert!(status.is_success(), "initialize answered {status}");
        let initialized = client
            .post(&json!({"jsonrpc": "2.0", "method": "notifications/initialized"}))
            .await;
        assert!(
            initialized.status().is_success(),
            "initialized notification answered {}",
            initialized.status()
        );
        client
    }

    async fn post(&self, body: &Value) -> reqwest::Response {
        let mut request = self
            .http
            .post(&self.url)
            .bearer_auth(&self.bearer)
            .header("accept", "application/json, text/event-stream")
            .header(headers::SESSION_ID, uuid::Uuid::new_v4().to_string())
            .header(headers::TRACE_ID, uuid::Uuid::new_v4().to_string())
            .header(headers::USER_ID, &self.user_id)
            .header(headers::AGENT_NAME, "docs-smoke")
            .header(headers::CONTEXT_ID, &self.context_id)
            .json(body);
        if !self.session.is_empty() {
            request = request.header(MCP_SESSION, &self.session);
        }
        request
            .send()
            .await
            .expect("reach the documents MCP server")
    }

    async fn request(&mut self, method: &str, params: Value) -> Value {
        self.next_id += 1;
        let id = self.next_id;
        let response = self
            .post(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}))
            .await;
        let status = response.status();
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        let body = response.text().await.expect("read the MCP response");
        assert!(status.is_success(), "{method} answered {status}: {body}");
        let reply = read_body(&content_type, &body);
        assert!(reply.get("error").is_none(), "{method} failed: {reply}");
        reply["result"].clone()
    }

    pub(super) async fn list_tools(&mut self) -> Value {
        self.request("tools/list", json!({})).await
    }

    pub(super) async fn call(&mut self, tool: &str, arguments: Value) -> ToolResult {
        let raw = self
            .request("tools/call", json!({"name": tool, "arguments": arguments}))
            .await;
        ToolResult {
            is_error: raw["isError"].as_bool().unwrap_or(false),
            payload: payload(&raw),
            raw,
        }
    }

    pub(super) async fn ok(&mut self, tool: &str, arguments: Value) -> ToolResult {
        let result = self.call(tool, arguments).await;
        assert!(!result.is_error, "{tool} refused: {}", result.raw);
        result
    }
}
