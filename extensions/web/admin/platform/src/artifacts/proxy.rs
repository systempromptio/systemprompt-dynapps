//! MCP sessions through this instance's own MCP proxy, as one person.
//!
//! The broker is an MCP client of `/api/v1/mcp/{server}/mcp` on the API's
//! internal URL, the way core's own MCP client reaches a managed server, and
//! it presents the person's own token as the bearer (the viewer's web
//! session for a page's call, the agent's own bearer when the `documents`
//! server checks a declaration). So the proxy
//! does what it does for any caller: it authenticates the token, checks the
//! server's scopes, resolves a connector's per-user OAuth for an external
//! server, and signs the hop token that carries the viewer to a managed
//! server, whose host checks the server's access rules and the tool's
//! `access:` and records the call. Nothing is decided here that the proxy
//! or the server decides.
//!
//! The exchange is the streamable-HTTP handshake at the last MCP protocol
//! version that still has `initialize` (`PROTOCOL_VERSION`): `initialize`
//! and `notifications/initialized`, then `tools/list` (every page) and
//! `tools/call` as the caller needs them, then `DELETE` of the session.
//! `answer` reads what came back into the broker's codes.

use std::time::Duration;

use reqwest::header::{ACCEPT, CONTENT_TYPE};
// JSON: MCP-protocol boundary, JSON-RPC messages carry schema-less payloads.
use serde_json::{Value, json};
use systemprompt::identifiers::{McpServerId, McpToolName};
use systemprompt::models::ApiPaths;
use systemprompt::models::net::{LoopbackPolicy, validate_outbound_url};

use super::answer::{error_key, find_reply, refuse_status, rpc_result, unavailable, unreadable};
use super::error::{ArtifactCallCode, ArtifactCallError};

const PROTOCOL_VERSION: &str = "2025-11-25";
const SESSION_HEADER: &str = "mcp-session-id";
const PROTOCOL_HEADER: &str = "mcp-protocol-version";
const CALL_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_ANSWER_BYTES: usize = 8 * 1024 * 1024;
const MAX_REFUSAL_BYTES: usize = 16 * 1024;
const MAX_LIST_PAGES: usize = 20;

#[derive(Debug, Clone)]
pub(crate) struct LoopbackProxy {
    http: reqwest::Client,
    base: String,
}

pub(crate) struct Session<'a> {
    proxy: &'a LoopbackProxy,
    url: String,
    bearer: &'a str,
    id: Option<String>,
    version: String,
    next: u64,
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

    pub(crate) async fn open<'a>(
        &'a self,
        server: &McpServerId,
        bearer: &'a str,
    ) -> Result<Session<'a>, ArtifactCallError> {
        let url = validate_outbound_url(
            &format!(
                "{}{}",
                self.base,
                ApiPaths::mcp_server_endpoint(server.as_str())
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
            bearer,
            id: None,
            version: PROTOCOL_VERSION.to_owned(),
            next: 0,
        };
        match session.handshake().await {
            Ok(()) => Ok(session),
            Err(error) => {
                session.close().await;
                Err(error)
            },
        }
    }
}

impl Session<'_> {
    async fn handshake(&mut self) -> Result<(), ArtifactCallError> {
        let initialize = json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": { "name": "systemprompt-dynapps-artifacts", "version": env!("CARGO_PKG_VERSION") },
        });
        let (session, handshake) = self.request("initialize", initialize).await?;
        self.id = session;
        if let Some(version) = handshake.get("protocolVersion").and_then(Value::as_str) {
            version.clone_into(&mut self.version);
        }
        let initialized = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
        self.post(&initialized, None).await.map(|_| ())
    }

    // JSON: MCP-protocol boundary, each tool as the server lists it.
    pub(crate) async fn tools(&mut self) -> Result<Vec<Value>, ArtifactCallError> {
        let mut tools = Vec::new();
        let mut cursor: Option<String> = None;
        for _ in 0..MAX_LIST_PAGES {
            let params = cursor.map_or_else(|| json!({}), |cursor| json!({ "cursor": cursor }));
            let (_, page) = self.request("tools/list", params).await?;
            if let Some(listed) = page.get("tools").and_then(Value::as_array) {
                tools.extend(listed.iter().cloned());
            }
            cursor = page
                .get("nextCursor")
                .and_then(Value::as_str)
                .map(str::to_owned);
            if cursor.is_none() {
                return Ok(tools);
            }
        }
        tracing::warn!("an MCP server listed more tool pages than a page may read");
        Ok(tools)
    }

    // JSON: MCP-protocol boundary, the tool's arguments and result.
    pub(crate) async fn call(
        &mut self,
        tool: &McpToolName,
        arguments: Value,
    ) -> Result<Value, ArtifactCallError> {
        let params = json!({ "name": tool.as_str(), "arguments": arguments });
        self.request("tools/call", params)
            .await
            .map(|(_, result)| result)
    }

    // JSON: MCP-protocol boundary, one JSON-RPC request and its result.
    async fn request(
        &mut self,
        method: &str,
        params: Value,
    ) -> Result<(Option<String>, Value), ArtifactCallError> {
        self.next += 1;
        let id = self.next;
        let message = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        let (session, answer) = self.post(&message, Some(id)).await?;
        answer
            .map(|result| (session, result))
            .ok_or_else(unreadable)
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
        if !response.status().is_success() {
            let refusal = refusal_body(&mut response).await;
            refuse_status(response.status(), error_key(&refusal).as_deref())?;
        }
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

    pub(crate) async fn close(self) {
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

async fn refusal_body(response: &mut reqwest::Response) -> Vec<u8> {
    let mut body = Vec::new();
    while body.len() < MAX_REFUSAL_BYTES {
        match response.chunk().await {
            Ok(Some(chunk)) => body.extend_from_slice(&chunk),
            Ok(None) => break,
            Err(error) => {
                tracing::debug!(%error, "the MCP proxy's refusal broke off");
                break;
            },
        }
    }
    body
}
