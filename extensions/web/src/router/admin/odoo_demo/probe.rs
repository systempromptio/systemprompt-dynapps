//! The demo's closing proof that a revoked grant refuses a delete.
//!
//! Once the grant is revoked, the service account attempts a real
//! `crm_lead_delete` through the Odoo MCP server, with a short-lived MCP token
//! minted for it, and the refusal is asserted and recorded in the activity
//! feed.
//!
//! The call goes over the same HTTP endpoint a farm run uses, so the server's
//! own destructive-role gate decides it and writes its governance row. Nothing
//! in Odoo is read or changed: the gate refuses before any credential is
//! resolved.

use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use systemprompt::agent_runs::credentials::{
    IssueRunCredentials, RunCredentialLimits, RunCredentialService,
};
use systemprompt::agent_runs::repository::RunCredentialRepositories;
use systemprompt::database::Database;
use systemprompt::identifiers::UserId;
use systemprompt::prelude::PgPool;
use systemprompt_web_admin_core::activity::{self, ActivityAction, ActivityCategory, NewActivity};

const TOOL: &str = "crm_lead_delete";
const PROBE_LEAD_ID: i64 = 1;
const TTL: Duration = Duration::from_mins(5);
const PROTOCOL_VERSION: &str = "2025-06-18";
const ACCEPT: &str = "application/json, text/event-stream";
const SESSION_HEADER: &str = "mcp-session-id";
const PURPOSE: &str = "odoo-demo:post-revoke-probe";

#[derive(Debug, Clone, Serialize)]
pub(super) struct PostRevokeProbe {
    pub tool: &'static str,
    pub refused: bool,
    pub detail: String,
}

// Why: only an MCP `isError` result naming the role proves the revoke; a
// transport fault or a bad-argument error does not.
// JSON: MCP JSON-RPC response at the protocol boundary.
#[must_use]
pub(super) fn is_role_refusal(reply: &serde_json::Value) -> bool {
    let result = &reply["result"];
    let is_error = result["isError"].as_bool() == Some(true);
    let text = result["content"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|block| block["text"].as_str())
        .collect::<Vec<_>>()
        .join(" ");
    is_error && text.starts_with("Refused:") && text.contains(super::DEMO_ROLE)
}

// JSON: MCP streamable HTTP answers either as JSON or as one SSE `data:` frame.
fn parse_reply(body: &str) -> Option<serde_json::Value> {
    serde_json::from_str(body).ok().or_else(|| {
        body.lines()
            .filter_map(|line| line.strip_prefix("data:"))
            .filter_map(|data| serde_json::from_str::<serde_json::Value>(data.trim()).ok())
            .find(|value| value.get("id").is_some())
    })
}

struct McpSession<'a> {
    http: reqwest::Client,
    url: String,
    token: &'a str,
    session: Option<String>,
}

impl McpSession<'_> {
    // JSON: MCP JSON-RPC request at the protocol boundary.
    async fn post(&mut self, body: serde_json::Value) -> Result<String, String> {
        let mut request = self
            .http
            .post(&self.url)
            .bearer_auth(self.token)
            .header("accept", ACCEPT)
            .header("mcp-protocol-version", PROTOCOL_VERSION)
            .json(&body);
        if let Some(session) = &self.session {
            request = request.header(SESSION_HEADER, session);
        }
        let response = request.send().await.map_err(|e| e.to_string())?;
        if let Some(id) = response
            .headers()
            .get(SESSION_HEADER)
            .and_then(|v| v.to_str().ok())
        {
            self.session = Some(id.to_owned());
        }
        let status = response.status();
        let text = response.text().await.map_err(|e| e.to_string())?;
        if status.is_success() {
            Ok(text)
        } else {
            Err(format!("HTTP {status}: {text}"))
        }
    }
}

// JSON: MCP JSON-RPC reply at the protocol boundary.
async fn attempt_delete(url: String, token: &str) -> Result<serde_json::Value, String> {
    let mut mcp = McpSession {
        http: reqwest::Client::new(),
        url,
        token,
        session: None,
    };
    mcp.post(serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": { "name": "dynapps-odoo-demo", "version": "1" }
        }
    }))
    .await?;
    mcp.post(serde_json::json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }))
        .await?;
    let body = mcp
        .post(serde_json::json!({
            "jsonrpc": "2.0", "id": 2, "method": "tools/call",
            "params": {
                "name": TOOL,
                "arguments": {
                    "id": PROBE_LEAD_ID,
                    "reason": "Odoo demo: proving the delete is refused once the grant is revoked"
                }
            }
        }))
        .await?;
    parse_reply(&body).ok_or_else(|| format!("unreadable MCP reply: {body}"))
}

pub(super) struct ProbeTarget<'a> {
    pub pool: &'a Arc<PgPool>,
    pub account: &'a UserId,
    pub actor: &'a UserId,
    pub mcp_url: String,
}

pub(super) async fn run(target: ProbeTarget<'_>) -> PostRevokeProbe {
    let outcome = probe(&target).await;
    let probe = match outcome {
        Ok(reply) => PostRevokeProbe {
            tool: TOOL,
            refused: is_role_refusal(&reply),
            detail: reply["result"]["content"][0]["text"]
                .as_str()
                .map_or_else(|| reply.to_string(), ToOwned::to_owned),
        },
        Err(detail) => PostRevokeProbe {
            tool: TOOL,
            refused: false,
            detail,
        },
    };
    record(&target, &probe).await;
    probe
}

// JSON: MCP JSON-RPC reply at the protocol boundary.
async fn probe(target: &ProbeTarget<'_>) -> Result<serde_json::Value, String> {
    let issuer = systemprompt::manifest::Config::get()
        .map_err(|e| e.to_string())?
        .jwt_issuer
        .clone();
    let db = Arc::new(Database::from_pools(
        Arc::clone(target.pool),
        Some(Arc::clone(target.pool)),
    ));
    let service = RunCredentialService::new(RunCredentialRepositories::new(&db), issuer);
    let credentials = service
        .issue(IssueRunCredentials {
            user_id: target.account.clone(),
            name: "odoo-demo-post-revoke".to_owned(),
            ttl: TTL,
            scopes: Vec::new(),
            plugin_id: None,
            limits: RunCredentialLimits {
                max_budget_usd: None,
                max_requests: None,
            },
            purpose: Some(PURPOSE.to_owned()),
        })
        .await
        .map_err(|e| e.to_string())?;
    let result = match credentials.mcp_token.as_deref() {
        Some(token) => attempt_delete(target.mcp_url.clone(), token).await,
        None => Err("no MCP token was issued for the service account".to_owned()),
    };
    if let Err(error) = service.revoke(&credentials).await {
        tracing::warn!(target: "odoo_demo", %error, "post-revoke probe credentials not revoked");
    }
    result
}

async fn record(target: &ProbeTarget<'_>, probe: &PostRevokeProbe) {
    let verdict = if probe.refused {
        "refused"
    } else {
        "NOT refused"
    };
    if probe.refused {
        tracing::info!(target: "odoo_demo", account = target.account.as_str(),
            "post-revoke crm_lead_delete refused");
    } else {
        tracing::error!(target: "odoo_demo", account = target.account.as_str(),
            detail = %probe.detail, "post-revoke crm_lead_delete was not refused");
    }
    activity::record(
        target.pool,
        NewActivity {
            user_id: target.actor.clone(),
            category: ActivityCategory::UserManagement,
            action: ActivityAction::Rejected,
            entity: None,
            description: format!(
                "Odoo demo: after revoking {}, {} by {} was {verdict}",
                super::DEMO_ROLE,
                probe.tool,
                target.account.as_str()
            ),
            // JSON: JSONB column — the probe's verdict and the server's reply.
            metadata: serde_json::json!({
                "account": target.account.as_str(),
                "tool": probe.tool,
                "refused": probe.refused,
                "detail": probe.detail,
            }),
        },
    )
    .await;
}
