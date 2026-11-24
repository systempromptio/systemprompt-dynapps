//! One `tools/call` through this instance's own MCP proxy, as the viewer.
//!
//! The broker is an MCP client of `/api/v1/mcp/{server}/mcp` on the API's
//! internal URL, the way core's own MCP client reaches a managed server, and
//! it presents the viewer's own session token as the bearer. So the proxy
//! does what it does for any caller: it authenticates the token, checks the
//! server's scopes, resolves a connector's per-user OAuth for an external
//! server, and signs the hop token that carries the viewer to a managed
//! server, whose host checks the server's access rules and the tool's
//! `access:` and records the call. Nothing is decided here that the proxy
//! or the server decides.
//!
//! The exchange is the streamable-HTTP handshake at the last MCP protocol
//! version that still has `initialize` (`PROTOCOL_VERSION`): `initialize`,
//! `notifications/initialized`, `tools/call`, then `DELETE` of the session.
//! `answer` reads what came back into the broker's codes.

use std::time::Duration;

use reqwest::header::{ACCEPT, CONTENT_TYPE};
// JSON: MCP-protocol boundary, JSON-RPC messages carry schema-less payloads.
use serde_json::{Value, json};
use systemprompt::identifiers::{McpServerId, McpToolName};
use systemprompt::models::ApiPaths;
use systemprompt::models::net::{LoopbackPolicy, validate_outbound_url};

use super::answer::{find_reply, refuse_status, rpc_result, unavailable, unreadable};
use super::error::{ArtifactCallCode, ArtifactCallError};

const PROTOCOL_VERSION: &str = "2025-11-25";
const SESSION_HEADER: &str = "mcp-session-id";
const PROTOCOL_HEADER: &str = "mcp-protocol-version";
const CALL_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_ANSWER_BYTES: usize = 8 * 1024 * 1024;
const INITIALIZE_ID: u64 = 1;
const CALL_ID: u64 = 2;

#[derive(Debug, Clone)]
pub(crate) struct LoopbackProxy {
    http: reqwest::Client,
    base: String,
}

// JSON: MCP-protocol boundary, schema-less tool arguments and results.
pub(crate) struct ProxiedCall<'a> {
    pub(crate) server: &'a McpServerId,
    pub(crate) tool: &'a McpToolName,
    pub(crate) arguments: Value,
    pub(crate) bearer: &'a str,
}

struct Session<'a> {
    proxy: &'a LoopbackProxy,
    url: String,
    bearer: &'a str,
    id: Option<String>,
    version: String,
}

impl LoopbackProxy {
    pub(crate) fn new(api_internal_url: &str) -> Result<Self, reqwest::Error> {
        let http = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(CALL_TIMEOUT)
            .build()?;
        Ok(Self {
            http,
            base: api_internal_url.trim_end_matches('/').to_owned(),
        })
    }

    // JSON: MCP-protocol boundary, the tool result as core shapes it.
    pub(crate) async fn call_tool(
        &self,
        call: ProxiedCall<'_>,
    ) -> Result<Value, ArtifactCallError> {
        let url = validate_outbound_url(
            &format!(
                "{}{}",
                self.base,
                ApiPaths::mcp_server_endpoint(call.server.as_str())
            ),
            LoopbackPolicy::Allow,
        )
        .map_err(|error| {
            tracing::error!(%error, "the API's internal URL is not a loopback destination");
            ArtifactCallError::new(
                ArtifactCallCode::ServerUnavailable,
                "this instance cannot reach its own MCP proxy",
            )
        })?;
        let mut session = Session {
            proxy: self,
            url: url.to_string(),
            bearer: call.bearer,
            id: None,
            version: PROTOCOL_VERSION.to_owned(),
        };
        let answer = session.call(&call).await;
        session.close().await;
        answer
    }
}

impl Session<'_> {
    async fn call(&mut self, call: &ProxiedCall<'_>) -> Result<Value, ArtifactCallError> {
        let initialize = json!({
            "jsonrpc": "2.0",
            "id": INITIALIZE_ID,
            "method": "initialize",
            "params": {
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {},
                "clientInfo": { "name": "systemprompt-dynapps-artifacts", "version": env!("CARGO_PKG_VERSION") },
            },
        });
        let (session, handshake) = self.post(&initialize, Some(INITIALIZE_ID)).await?;
        self.id = session;
        if let Some(version) = handshake
            .as_ref()
            .and_then(|result| result.get("protocolVersion"))
            .and_then(Value::as_str)
        {
            version.clone_into(&mut self.version);
        }
        let initialized = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
        self.post(&initialized, None).await?;
        let request = json!({
            "jsonrpc": "2.0",
            "id": CALL_ID,
            "method": "tools/call",
            "params": { "name": call.tool.as_str(), "arguments": call.arguments },
        });
        let (_, answer) = self.post(&request, Some(CALL_ID)).await?;
        answer.ok_or_else(unreadable)
    }

    // JSON: MCP-protocol boundary, one JSON-RPC message out and its answer back.
    async fn post(
        &self,
        message: &Value,
        id: Option<u64>,
    ) -> Result<(Option<String>, Option<Value>), ArtifactCallError> {
        let mut request = self
            .proxy
            .http
            .post(&self.url)
            .bearer_auth(self.bearer)
            .header(ACCEPT, "application/json, text/event-stream")
            .header(PROTOCOL_HEADER, &self.version)
            .json(message);
        if let Some(session) = &self.id {
            request = request.header(SESSION_HEADER, session);
        }
        let mut response = request.send().await.map_err(|error| {
            tracing::warn!(%error, "the MCP proxy did not answer");
            unavailable()
        })?;
        refuse_status(response.status())?;
        let session = response
            .headers()
            .get(SESSION_HEADER)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let Some(id) = id else {
            return Ok((session, None));
        };
        let sse = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.starts_with("text/event-stream"));
        let mut body = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|error| {
            tracing::warn!(%error, "the MCP proxy's answer broke off");
            unavailable()
        })? {
            if body.len() + chunk.len() > MAX_ANSWER_BYTES {
                return Err(ArtifactCallError::new(
                    ArtifactCallCode::ToolError,
                    "the tool's result is larger than a page may receive",
                ));
            }
            body.extend_from_slice(&chunk);
            let event_ended = chunk.windows(2).any(|pair| pair == b"\n\n");
            if sse
                && event_ended
                && let Some(reply) = find_reply(&body, true, id)
            {
                return rpc_result(reply).map(|result| (session, Some(result)));
            }
        }
        let reply = find_reply(&body, sse, id).ok_or_else(unreadable)?;
        rpc_result(reply).map(|result| (session, Some(result)))
    }

    async fn close(&self) {
        let Some(session) = &self.id else {
            return;
        };
        let closed = self
            .proxy
            .http
            .delete(&self.url)
            .bearer_auth(self.bearer)
            .header(SESSION_HEADER, session)
            .header(PROTOCOL_HEADER, &self.version)
            .send()
            .await;
        if let Err(error) = closed {
            tracing::debug!(%error, "the MCP session was not closed; the proxy's cleanup job ends it");
        }
    }
}
