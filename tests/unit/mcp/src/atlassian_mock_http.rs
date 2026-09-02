//! Wiremock harness for the Atlassian source connectors.
//!
//! Mirrors core's `mock_http` fixtures for the AI provider drivers: each
//! helper starts a fresh [`MockServer`], registers a canned response, and the
//! client is pointed at it. Here no `with_endpoint` constructor is needed —
//! the connector reads its base URL from the caller's own credentials, which
//! is exactly the seam a test wants.

use serde_json::{Value, json};
use systemprompt_mcp_knowledge_bank::config::{ChunkStrategy, SourceConfig, SourceKind};
use systemprompt_mcp_knowledge_bank::downstream::AtlassianCredentials;
use systemprompt_mcp_knowledge_bank::ingest::sources::AtlassianClient;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub fn client_for(server: &MockServer) -> AtlassianClient {
    AtlassianClient::new(AtlassianCredentials {
        base_url: server.uri(),
        email: "caller@example.com".to_owned(),
        api_token: "token-value".to_owned(),
    })
    .expect("the client builds against the mock server")
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

// Jira's search endpoint, answering each mounted body in turn so a paged run
// can be driven. `up_to` is how many responses this mock will serve.
pub async fn jira_search(bodies: Vec<Value>) -> MockServer {
    let server = MockServer::start().await;
    for (index, body) in bodies.into_iter().enumerate() {
        Mock::given(method("POST"))
            .and(path("/rest/api/3/search/jql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .up_to_n_times(1)
            .with_priority(u8::try_from(index + 1).unwrap_or(u8::MAX))
            .mount(&server)
            .await;
    }
    server
}

pub async fn jira_search_failing(status: u16, detail: &str) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/search/jql"))
        .respond_with(ResponseTemplate::new(status).set_body_string(detail.to_owned()))
        .mount(&server)
        .await;
    server
}

// One Jira issue, in the shape the v3 search endpoint returns it.
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
            "description": { "type": "doc", "content": [] },
            "comment": {
                "comments": [{
                    "author": { "displayName": "Dana Scully" },
                    "created": "2026-08-20T11:00:00.000+0000",
                    "body": { "type": "doc", "content": [] }
                }]
            }
        },
        "renderedFields": {
            "description": "<p>The ingest pipeline must grant each document to its reader.</p>",
            "comment": {
                "comments": [{
                    "author": { "displayName": "Dana Scully" },
                    "created": "2026-08-20T11:00:00.000+0000",
                    "body": "<p>Agreed — grants go in the same transaction.</p>"
                }]
            }
        }
    })
}

// Confluence: the space lookup, then one page of results, then each page's
// body. `space_id` is what the lookup resolves the key to.
pub async fn confluence_space(space_key: &str, space_id: &str, pages: Value) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces"))
        .and(query_param("keys", space_key))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{ "id": space_id, "key": space_key }]
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/wiki/api/v2/spaces/{space_id}/pages")))
        .respond_with(ResponseTemplate::new(200).set_body_json(pages))
        .mount(&server)
        .await;
    server
}

pub async fn confluence_space_missing(space_key: &str) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces"))
        .and(query_param("keys", space_key))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "results": [] })))
        .mount(&server)
        .await;
    server
}

pub fn page(id: &str, title: &str, html: &str, updated: &str) -> Value {
    json!({
        "id": id,
        "title": title,
        "status": "current",
        "spaceId": "1001",
        "body": { "storage": { "value": html, "representation": "storage" } },
        "version": { "number": 4, "createdAt": updated },
        "_links": { "webui": format!("/spaces/ENG/pages/{id}") }
    })
}
