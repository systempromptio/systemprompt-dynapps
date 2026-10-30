//! The lifecycle half of the agent's run over the binary: a note created
//! from the compiled-in `dynapps-note` provider, which proves the binary
//! links the crate that registers it, and the memo trashed and brought back:
//! the agent's `documents_trash` refused as `agent_forbidden`, the owner's
//! REST trash and untrash, and the agent's read in between refused as
//! `trashed`.

use reqwest::StatusCode;
use serde_json::json;
use systemprompt::identifiers::DocumentId;

use super::flow::read;
use super::mcp::McpClient;
use super::rest::Rest;

pub(super) async fn note_from_provider(mcp: &mut McpClient) -> DocumentId {
    let created = mcp
        .ok(
            "documents_create",
            json!({
                "template_id": "dynapps-note",
                "name": format!("docs-smoke-note-{}", uuid::Uuid::new_v4().simple()),
                "params": {"title": "Smoke note"},
            }),
        )
        .await;
    assert_eq!(created.payload["format"], "markdown", "{}", created.raw);
    let document = DocumentId::new(
        created.payload["document_id"]
            .as_str()
            .expect("create names the document"),
    );
    let (_, shown) = read(mcp, &document).await;
    let texts: Vec<&str> = shown.iter().map(|l| l.text.as_str()).collect();
    assert!(
        texts.contains(&"# Smoke note"),
        "the title fills the heading: {texts:?}"
    );
    assert!(
        texts.iter().any(|text| text.contains("Dynapps")),
        "the byline is kept: {texts:?}"
    );
    document
}

pub(super) async fn trash_and_untrash(
    mcp: &mut McpClient,
    rest: &Rest,
    owner: &str,
    document: &DocumentId,
) {
    let forbidden = mcp
        .call("documents_trash", json!({"document_id": document}))
        .await;
    assert!(forbidden.is_error, "an agent trashes: {}", forbidden.raw);
    assert_eq!(
        forbidden.payload["code"], "agent_forbidden",
        "{}",
        forbidden.raw
    );
    let path = format!("/api/v1/documents/{}", document.as_str());
    let trashed = rest.json("DELETE", &path, owner, &json!({})).await;
    assert_eq!(trashed.status, StatusCode::NO_CONTENT, "{}", trashed.body);
    let refused = mcp
        .call(
            "documents_read",
            json!({"document_id": document, "view": "text"}),
        )
        .await;
    assert!(
        refused.is_error,
        "a trashed document reads: {}",
        refused.raw
    );
    assert_eq!(refused.payload["code"], "trashed", "{}", refused.raw);
    assert_eq!(refused.payload["trashed"], true, "{}", refused.raw);
    let back = rest
        .json("POST", &format!("{path}/untrash"), owner, &json!({}))
        .await;
    assert_eq!(back.status, StatusCode::OK, "{}", back.body);
    let (_, shown) = read(mcp, document).await;
    assert!(
        shown.iter().any(|line| line.text == "alpha by agent"),
        "the head survives the trash: {shown:?}"
    );
}
