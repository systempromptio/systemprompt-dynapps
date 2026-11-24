//! The page's manifest, declared by the agent that writes the page: the
//! extension's `documents` server takes `capabilities` on a create and an
//! update of an HTML document, checks every server and tool against the
//! instance (as the agent's own person sees them) and the operator's
//! live-pages policy, and records the declaration. The shell reads it back
//! with the viewer's state.

use serde_json::{Value, json};
use systemprompt::identifiers::{DocumentId, UserId};

use super::fake::{ECHO, OFF, WRITE};
use super::stack::{As, DOCUMENTS, SERVER, Stack};
use crate::docs_smoke::mcp::McpClient;

pub(super) fn declared(tools: &[&str]) -> Value {
    json!({
        "mcp": { "servers": [{ "server": SERVER, "tools": tools }] },
        "artifact": {},
        "user": {},
    })
}

pub(super) async fn state(stack: &Stack, cookie: &str, document: &DocumentId) -> Value {
    let reply = stack
        .send(
            reqwest::Method::GET,
            &format!("/api/v1/artifacts/{}/permissions", document.as_str()),
            As::Cookie(cookie),
            None,
        )
        .await;
    assert_eq!(
        reply.status, 200,
        "the viewer reads the page's state: {}",
        reply.body
    );
    reply.json()
}

async fn refused(agent: &mut McpClient, document: &DocumentId, capabilities: Value, why: &str) {
    let outcome = agent
        .outcome(
            "documents_update",
            json!({ "document_id": document.as_str(), "capabilities": capabilities }),
        )
        .await;
    assert!(
        outcome.is_err(),
        "{why} is refused, got {:?}",
        outcome.map(|ok| ok.raw)
    );
}

pub(super) async fn the_agent_declares_a_page(
    stack: &Stack,
    pool: &sqlx::PgPool,
    owner: &UserId,
    cookie: &str,
) -> DocumentId {
    let mut agent = stack.agent(pool, owner).await;
    let listed = agent.list_tools().await;
    for tool in [
        "documents_create",
        "documents_update",
        "documents_edit",
        "documents_import",
    ] {
        let schema = listed["tools"]
            .as_array()
            .and_then(|tools| tools.iter().find(|entry| entry["name"] == tool))
            .unwrap_or_else(|| panic!("{tool} is listed: {listed}"));
        assert!(
            schema["inputSchema"]["properties"]["capabilities"].is_object(),
            "{tool} takes `capabilities`: {schema}"
        );
    }

    let created = agent
        .ok(
            "documents_create",
            json!({
                "template_id": "dynapps-html",
                "name": "live-page",
                "params": { "title": "Live" },
                "capabilities": declared(&[ECHO, WRITE]),
            }),
        )
        .await;
    let document = DocumentId::new(
        created.payload["document_id"]
            .as_str()
            .unwrap_or_else(|| panic!("create names the document: {}", created.raw)),
    );
    let read = state(stack, cookie, &document).await;
    assert_eq!(
        read["capabilities"],
        declared(&[ECHO, WRITE]),
        "the create's declaration is the page's manifest: {read}"
    );
    assert!(
        read["contract_version"]
            .as_str()
            .is_some_and(|v| !v.is_empty()),
        "the manifest names its runtime contract: {read}"
    );

    refused(
        &mut agent,
        &document,
        json!({ "mcp": { "servers": [{ "server": "no-such-server", "tools": [ECHO] }] } }),
        "an unknown server",
    )
    .await;
    refused(
        &mut agent,
        &document,
        declared(&["no_such_tool"]),
        "an unknown tool",
    )
    .await;
    refused(&mut agent, &document, declared(&[]), "an empty tool list").await;
    refused(
        &mut agent,
        &document,
        declared(&[OFF]),
        "a tool the policy leaves out",
    )
    .await;
    refused(
        &mut agent,
        &document,
        json!({ "mcp": { "servers": [{ "server": DOCUMENTS, "tools": ["documents_list"] }] } }),
        "a server the policy leaves out",
    )
    .await;
    refused(
        &mut agent,
        &document,
        json!({ "db": {} }),
        "a capability this instance does not offer",
    )
    .await;
    assert_eq!(
        state(stack, cookie, &document).await["capabilities"],
        declared(&[ECHO, WRITE]),
        "a refused declaration leaves the manifest as it was"
    );

    agent
        .ok(
            "documents_update",
            json!({ "document_id": document.as_str(), "capabilities": {} }),
        )
        .await;
    assert!(
        state(stack, cookie, &document).await["capabilities"].is_null(),
        "`{{}}` clears the manifest"
    );
    agent
        .ok(
            "documents_update",
            json!({ "document_id": document.as_str(), "capabilities": declared(&[ECHO, WRITE]) }),
        )
        .await;
    agent
        .ok(
            "documents_update",
            json!({ "document_id": document.as_str(), "name": "live-page-renamed" }),
        )
        .await;
    assert_eq!(
        state(stack, cookie, &document).await["capabilities"],
        declared(&[ECHO, WRITE]),
        "an update without `capabilities` keeps the manifest"
    );
    document
}
