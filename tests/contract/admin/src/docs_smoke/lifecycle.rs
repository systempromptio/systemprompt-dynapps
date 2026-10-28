//! The lifecycle half of the agent's run over the binary: a note created
//! from the compiled-in `dynapps-note` provider, which proves the binary
//! links the crate that registers it, and the memo trashed and brought back
//! with `documents_trash` and `documents_untrash`, a read in between refused
//! as `trashed`.

use serde_json::json;
use systemprompt::identifiers::DocumentId;

use super::flow::read;
use super::mcp::McpClient;

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

pub(super) async fn trash_and_untrash(mcp: &mut McpClient, document: &DocumentId) {
    let trashed = mcp
        .ok("documents_trash", json!({"document_id": document}))
        .await;
    assert_eq!(trashed.payload["trashed"], true, "{}", trashed.raw);
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
    let back = mcp
        .ok("documents_untrash", json!({"document_id": document}))
        .await;
    assert_eq!(
        back.payload["document_id"],
        document.as_str(),
        "{}",
        back.raw
    );
    assert!(back.payload.get("trashed").is_none(), "{}", back.raw);
    let (_, shown) = read(mcp, document).await;
    assert!(
        shown.iter().any(|line| line.text == "alpha by agent"),
        "the head survives the trash: {shown:?}"
    );
}
