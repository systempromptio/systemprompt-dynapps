//! The Atlassian MCP client used for indexing.
//!
//! Every read goes through this instance's own gateway route for the hosted
//! Atlassian MCP (`/api/v1/mcp/atlassian`) carrying the *caller's* systemprompt
//! bearer. Core swaps that bearer for the Atlassian token the caller banked
//! when they connected their account on `/admin/profile`, so this crate never
//! holds an Atlassian credential, every upstream read is governed and audited
//! like any other MCP call, and Atlassian enforces the caller's permissions on
//! each of them — which is what makes "grant everything fetched to the
//! caller" correct: a page this client cannot see is a page the caller
//! cannot see.

use crate::config::AtlassianSettings;
use crate::error::KnowledgeBankError;
use rmcp::model::{CallToolRequestParams, ClientCapabilities, ClientInfo, Implementation};
use rmcp::service::RunningService;
use rmcp::transport::streamable_http_client::{
    StreamableHttpClientTransport, StreamableHttpClientTransportConfig,
};
use rmcp::{RoleClient, ServiceExt};
// JSON: protocol boundary. The Atlassian MCP answers each tool with the
// product's own REST payload wrapped in `{"data": …}`; the connectors reach
// for the handful of fields they need by pointer rather than modelling a
// schema Atlassian varies per tool and per view.
use serde_json::{Value, json};
use std::time::Duration;
use systemprompt::config::ProfileBootstrap;
use systemprompt::mcp::services::client::HttpClientWithContext;
use systemprompt::models::execution::context::RequestContext;

pub const GATEWAY_SERVER: &str = "atlassian";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
const CALL_TIMEOUT: Duration = Duration::from_secs(90);
const RESOURCES_TOOL: &str = "getAccessibleAtlassianResources";

/// The connected Atlassian MCP, bound to one caller and one site.
pub struct AtlassianMcp {
    client: RunningService<RoleClient, ClientInfo>,
    cloud_id: String,
    site_url: String,
}

impl std::fmt::Debug for AtlassianMcp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AtlassianMcp")
            .field("cloud_id", &self.cloud_id)
            .field("site_url", &self.site_url)
            .finish_non_exhaustive()
    }
}

impl AtlassianMcp {
    pub async fn connect(
        bearer: &str,
        context: &RequestContext,
        settings: &AtlassianSettings,
    ) -> Result<Self, KnowledgeBankError> {
        // Why: without the active profile there is no gateway port to reach,
        // which to the caller is Atlassian being unavailable, not a bug.
        // lint-ok: error-adapt
        let profile = ProfileBootstrap::get().map_err(|e| {
            KnowledgeBankError::AtlassianUnavailable(format!("active profile unavailable: {e}"))
        })?;
        let url = format!(
            "http://127.0.0.1:{}/api/v1/mcp/{GATEWAY_SERVER}/mcp",
            profile.server.port
        );
        Self::connect_gateway(&url, bearer, context, settings).await
    }

    // Why: the gateway URL is a parameter so the connectors can be proven
    // against a fake gateway; `connect` is the production wrapper.
    pub async fn connect_gateway(
        url: &str,
        bearer: &str,
        context: &RequestContext,
        settings: &AtlassianSettings,
    ) -> Result<Self, KnowledgeBankError> {
        if bearer.trim().is_empty() {
            return Err(KnowledgeBankError::AtlassianUnavailable(
                "no caller credential on this connection".to_owned(),
            ));
        }
        let config = StreamableHttpClientTransportConfig::with_uri(url.to_owned())
            .auth_header(bearer.to_owned());
        let transport = StreamableHttpClientTransport::with_client(
            HttpClientWithContext::new(context.clone()),
            config,
        );
        let info = ClientInfo::new(
            ClientCapabilities::default(),
            Implementation::new("astound-knowledge-bank", env!("CARGO_PKG_VERSION")),
        );
        let client = tokio::time::timeout(CONNECT_TIMEOUT, info.serve(transport))
            .await
            .map_err(|_elapsed| {
                KnowledgeBankError::AtlassianUnavailable(
                    "connecting to the Atlassian MCP through this instance timed out".to_owned(),
                )
            })?
            .map_err(|e| {
                KnowledgeBankError::AtlassianUnavailable(format!(
                    "the Atlassian MCP could not be reached through this instance: {e}"
                ))
            })?;
        let mut connected = Self {
            client,
            cloud_id: String::new(),
            site_url: settings.base_url.trim().trim_end_matches('/').to_owned(),
        };
        connected.cloud_id = connected
            .resolve_cloud_id(settings.cloud_id.as_deref())
            .await?;
        Ok(connected)
    }

    // Why: every Atlassian tool wants the site's cloud id and the caller's
    // account may reach several sites. One configured id wins; otherwise a
    // single reachable site is unambiguous, and anything else is a
    // configuration question, not a guess.
    async fn resolve_cloud_id(
        &self,
        configured: Option<&str>,
    ) -> Result<String, KnowledgeBankError> {
        let payload = self.call(RESOURCES_TOOL, json!({})).await.map_err(|e| {
            KnowledgeBankError::AtlassianUnavailable(format!(
                "the connected Atlassian account could not be read: {e}"
            ))
        })?;
        let ids: Vec<String> = payload
            .get("resources")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|r| json_str(r, "/cloudId").or_else(|| json_str(r, "/id")))
            .map(str::to_owned)
            .collect();
        if let Some(wanted) = configured.map(str::trim).filter(|c| !c.is_empty()) {
            return if ids.iter().any(|id| id == wanted) {
                Ok(wanted.to_owned())
            } else {
                Err(KnowledgeBankError::AtlassianUnavailable(format!(
                    "your connected Atlassian account cannot reach site {wanted}"
                )))
            };
        }
        match ids.as_slice() {
            [] => Err(KnowledgeBankError::AtlassianUnavailable(
                "your connected Atlassian account reaches no site".to_owned(),
            )),
            [only] => Ok(only.clone()),
            many => Err(KnowledgeBankError::Config(format!(
                "the connected Atlassian account reaches {} sites ({}); set atlassian.cloud_id to the one to index",
                many.len(),
                many.join(", ")
            ))),
        }
    }

    #[must_use]
    pub fn cloud_id(&self) -> &str {
        &self.cloud_id
    }

    #[must_use]
    pub fn site_url(&self) -> &str {
        &self.site_url
    }

    pub async fn list_tools(&self) -> Result<Vec<rmcp::model::Tool>, KnowledgeBankError> {
        // Why: rmcp's service error is foreign and variant-less for our
        // purposes; the text is the whole diagnostic.
        // lint-ok: error-adapt
        self.client.list_all_tools().await.map_err(|e| {
            KnowledgeBankError::Backend(format!("Atlassian tool discovery failed: {e}"))
        })
    }

    pub async fn call(&self, name: &str, args: Value) -> Result<Value, KnowledgeBankError> {
        let mut params = CallToolRequestParams::new(name.to_owned());
        params.arguments = args.as_object().cloned();
        let response = tokio::time::timeout(CALL_TIMEOUT, self.client.call_tool(params))
            .await
            .map_err(|_elapsed| KnowledgeBankError::Backend(format!("atlassian {name} timed out")))?
            // Why: rmcp's service error names neither the tool nor the site;
            // the per-source report the caller reads has to.
            // lint-ok: error-adapt
            .map_err(|e| KnowledgeBankError::Backend(format!("atlassian {name} failed: {e}")))?;
        let text = response
            .content
            .iter()
            .find_map(|c| c.as_text().map(|t| t.text.clone()));
        if response.is_error == Some(true) {
            let detail: String = text.unwrap_or_default().chars().take(300).collect();
            return Err(KnowledgeBankError::Backend(format!(
                "atlassian {name} rejected the call: {detail}"
            )));
        }
        let value = match response.structured_content {
            Some(data) => data,
            None => text
                .as_deref()
                .and_then(|t| serde_json::from_str::<Value>(t).ok())
                .ok_or_else(|| {
                    KnowledgeBankError::Backend(format!("atlassian {name} returned no JSON"))
                })?,
        };
        Ok(unwrap_data(value))
    }
}

// Why: the hosted server wraps every product payload in `{"data": …}`; the
// connectors read the payload, not the envelope.
fn unwrap_data(value: Value) -> Value {
    match value {
        Value::Object(mut map) if map.contains_key("data") => {
            map.remove("data").unwrap_or(Value::Null)
        },
        other => other,
    }
}

#[must_use]
pub fn json_str<'a>(value: &'a Value, pointer: &str) -> Option<&'a str> {
    value.pointer(pointer).and_then(Value::as_str)
}

// Why: Atlassian's two products do not agree on how to write a timestamp.
// Confluence answers in RFC 3339 (`...Z` / `+00:00`); Jira Cloud answers in
// ISO 8601 basic form, whose offset carries no colon
// (`2026-08-30T14:00:00.000+0000`), which `parse_from_rfc3339` rejects.
#[must_use]
pub fn json_date(value: &Value, pointer: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    json_str(value, pointer)
        .and_then(parse_atlassian_timestamp)
        .map(|d| d.with_timezone(&chrono::Utc))
}

fn parse_atlassian_timestamp(raw: &str) -> Option<chrono::DateTime<chrono::FixedOffset>> {
    chrono::DateTime::parse_from_rfc3339(raw)
        .or_else(|_| chrono::DateTime::parse_from_str(raw, "%Y-%m-%dT%H:%M:%S%.f%z"))
        .ok()
}

// Why: rich text arrives in whichever shape the tool's view chose — a
// Markdown string, an `{format, value}` body, an ADF document, or HTML. The
// index wants Markdown text and nothing else.
#[must_use]
pub fn text_of(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(s)) => {
            if looks_like_html(s) {
                crate::ingest::html::html_to_markdown(s)
            } else {
                s.clone()
            }
        },
        Some(Value::Object(map)) => {
            if let Some(inner) = map.get("value") {
                return text_of(Some(inner));
            }
            if map.get("type").and_then(Value::as_str) == Some("doc") {
                let mut out = String::new();
                adf_text(&Value::Object(map.clone()), &mut out);
                return out.trim().to_owned();
            }
            String::new()
        },
        _ => String::new(),
    }
}

fn looks_like_html(text: &str) -> bool {
    text.contains("</") || text.contains("<p>") || text.contains("<br")
}

fn adf_text(node: &Value, out: &mut String) {
    if let Some(text) = node.get("text").and_then(Value::as_str) {
        out.push_str(text);
    }
    if let Some(children) = node.get("content").and_then(Value::as_array) {
        for child in children {
            adf_text(child, out);
        }
        if matches!(
            node.get("type").and_then(Value::as_str),
            Some("paragraph" | "heading" | "listItem" | "codeBlock")
        ) {
            out.push('\n');
        }
    }
}
