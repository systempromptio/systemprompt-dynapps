//! Wiremock fake of this instance's Atlassian MCP gateway route.
//!
//! The knowledge bank reads Atlassian as an MCP client of
//! `/api/v1/mcp/atlassian`, carrying the caller's bearer; core swaps that for
//! the connected account's token upstream. Here the whole route is faked:
//! `initialize` and the resources lookup are answered for the expected
//! bearer only, every tool the connectors call is mounted with the payload
//! the hosted server would wrap in `{"data": …}`, and anything else is
//! acknowledged like a notification so a wrong bearer or an unmounted tool
//! fails the way the real gateway would.

use serde_json::{Value, json};
use systemprompt::identifiers::{AgentName, ContextId, SessionId, TraceId};
use systemprompt::models::execution::context::RequestContext;
use systemprompt_mcp_knowledge_bank::config::{
    AtlassianSettings, ChunkStrategy, SourceConfig, SourceKind,
};
use systemprompt_mcp_knowledge_bank::error::KnowledgeBankError;
use systemprompt_mcp_knowledge_bank::ingest::sources::AtlassianMcp;
use wiremock::matchers::{body_partial_json, header, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub const BEARER: &str = "caller-bearer";
pub const SITE: &str = "https://astounddigital.atlassian.net";
pub const CLOUD_ID: &str = "site-a";

pub struct FakeGateway {
    pub server: MockServer,
}

impl FakeGateway {
    // A gateway whose connected account reaches exactly one site.
    pub async fn start() -> Self {
        Self::start_with_sites(&[CLOUD_ID]).await
    }

    pub async fn start_with_sites(sites: &[&str]) -> Self {
        let server = MockServer::start().await;
        // Notifications carry no id and want only an acknowledgement; this
        // is also what an unmatched call falls through to.
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(202))
            .with_priority(10)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(header("authorization", format!("Bearer {BEARER}").as_str()))
            .and(body_partial_json(json!({"method": "initialize"})))
            .respond_with(rpc_result(json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "atlassian-fake", "version": "1"}
            })))
            .with_priority(5)
            .mount(&server)
            .await;
        let gateway = Self { server };
        let resources: Vec<Value> = sites
            .iter()
            .map(|id| {
                json!({"cloudId": id, "products": [
                    {"id": "jira", "access": "read-only"},
                    {"id": "confluence", "access": "read-only"}
                ]})
            })
            .collect();
        gateway
            .on_call_at(
                "getAccessibleAtlassianResources",
                json!({}),
                json!({"resources": resources}),
                5,
            )
            .await;
        gateway
    }

    // Answer `tool` (when the arguments contain `args`) with `data`.
    pub async fn on_call(&self, tool: &str, args: Value, data: Value) {
        self.on_call_at(tool, args, data, 3).await;
    }

    // Why: wiremock picks the lowest priority number first, so a paged
    // listing mounts its continuation with a lower number than its first
    // page and matches on the cursor the first page handed out.
    pub async fn on_call_at(&self, tool: &str, args: Value, data: Value, priority: u8) {
        let text = json!({"data": data}).to_string();
        Mock::given(method("POST"))
            .and(header("authorization", format!("Bearer {BEARER}").as_str()))
            .and(body_partial_json(json!({
                "method": "tools/call",
                "params": {"name": tool, "arguments": args}
            })))
            .respond_with(rpc_result(json!({
                "content": [{"type": "text", "text": text}],
                "isError": false
            })))
            .with_priority(priority)
            .mount(&self.server)
            .await;
    }

    // Have `tool` fail the way the hosted server reports a refused read.
    pub async fn on_call_error(&self, tool: &str, message: &str) {
        Mock::given(method("POST"))
            .and(body_partial_json(json!({
                "method": "tools/call",
                "params": {"name": tool}
            })))
            .respond_with(rpc_result(json!({
                "content": [{"type": "text", "text": message}],
                "isError": true
            })))
            .with_priority(3)
            .mount(&self.server)
            .await;
    }

    pub async fn connect(&self) -> AtlassianMcp {
        self.try_connect(BEARER, settings())
            .await
            .expect("the fake gateway accepts the caller's bearer")
    }

    pub async fn try_connect(
        &self,
        bearer: &str,
        settings: AtlassianSettings,
    ) -> Result<AtlassianMcp, KnowledgeBankError> {
        AtlassianMcp::connect_gateway(&self.server.uri(), bearer, &context(), &settings).await
    }
}

fn rpc_result(result: Value) -> impl Fn(&wiremock::Request) -> ResponseTemplate + Send + Sync {
    move |request: &wiremock::Request| {
        let body: Value = serde_json::from_slice(&request.body).expect("rpc request body");
        ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": body["id"],
            "result": result
        }))
    }
}

pub fn settings() -> AtlassianSettings {
    AtlassianSettings {
        base_url: SITE.to_owned(),
        cloud_id: None,
        include_comments: true,
    }
}

pub fn context() -> RequestContext {
    RequestContext::new(
        SessionId::new("kb-session"),
        TraceId::new("kb-trace"),
        ContextId::new_unchecked("00000000-0000-4000-8000-000000000001"),
        AgentName::new("kb-test"),
    )
}

pub fn jira_source(projects: &[&str], jql: Option<&str>) -> SourceConfig {
    SourceConfig {
        kind: SourceKind::Jira,
        strategy: ChunkStrategy::Script,
        description: "Delivery tickets".to_owned(),
        archived: false,
        spaces: Vec::new(),
        projects: projects.iter().map(|p| (*p).to_owned()).collect(),
        jql: jql.map(str::to_owned),
        path: None,
        include: Vec::new(),
        exclude: Vec::new(),
        grants: Vec::new(),
    }
}

pub fn confluence_source(spaces: &[&str]) -> SourceConfig {
    SourceConfig {
        kind: SourceKind::Confluence,
        strategy: ChunkStrategy::Script,
        description: "Engineering spaces".to_owned(),
        archived: false,
        spaces: spaces.iter().map(|s| (*s).to_owned()).collect(),
        projects: Vec::new(),
        jql: None,
        path: None,
        include: Vec::new(),
        exclude: Vec::new(),
        grants: Vec::new(),
    }
}

// One Jira issue, in the shape the search tool's full view returns it, with
// the thread inline.
pub fn issue(key: &str, summary: &str, updated: &str) -> Value {
    json!({
        "key": key,
        "fields": {
            "summary": summary,
            "status": { "name": "In Progress" },
            "issuetype": { "name": "Story" },
            "priority": { "name": "High" },
            "assignee": { "displayName": "Dana Scully" },
            "reporter": { "displayName": "Fox Mulder" },
            "created": "2026-08-01T09:00:00.000+0000",
            "updated": updated,
            "labels": ["governance", "rag"],
            "description": "The ingest pipeline must grant each document to its reader.",
            "comment": {
                "comments": [{
                    "author": { "displayName": "Dana Scully" },
                    "created": "2026-08-20T11:00:00.000+0000",
                    "body": "Agreed — grants go in the same transaction."
                }]
            }
        }
    })
}

// Mount a space lookup and one page listing for it.
pub async fn confluence_space(gateway: &FakeGateway, key: &str, space_id: &str, pages: Vec<Value>) {
    gateway
        .on_call(
            "listConfluenceSpaces",
            json!({"keys": [key]}),
            json!({"results": [{"id": space_id, "key": key}], "_links": {"base": format!("{SITE}/wiki")}}),
        )
        .await;
    let listed: Vec<Value> = pages
        .iter()
        .map(|p| json!({"id": p["id"], "title": p["title"], "type": "page", "status": "current", "spaceId": space_id}))
        .collect();
    gateway
        .on_call(
            "listConfluenceContent",
            json!({"spaceId": space_id}),
            json!({"results": listed}),
        )
        .await;
    for page in pages {
        let id = page["id"].as_str().expect("page id").to_owned();
        gateway
            .on_call("getConfluenceContent", json!({"content_id": id}), page)
            .await;
    }
}

// One Confluence page as `getConfluenceContent` (detail: full) returns it.
pub fn page(id: &str, title: &str, markdown: &str, updated: &str) -> Value {
    json!({
        "id": id,
        "type": "page",
        "title": title,
        "status": "current",
        "spaceId": "1001",
        "detail": "full",
        "body": { "format": "markdown", "value": markdown },
        "metadata": { "version": { "number": 4, "createdAt": updated } }
    })
}
