//! The agent's editing half of the run, every step a `documents_*` tool call
//! to the running binary: the memo in one create, a stale guard refused, a
//! one-op edit, a find-and-replace plus the next paragraph in one edit, a
//! Markdown append and a three-revision history. Each step's effect is read
//! back from the document service. The helpers here read, target and edit
//! for the other steps too.

use serde_json::{Value, json};
use systemprompt::files::documents::DocumentService;
use systemprompt::identifiers::{DocumentId, DocumentNodeId, TraceId};
use systemprompt::models::auth::AuthenticatedUser;
use systemprompt::models::documents::{ContentAs, EditOp, NodeTarget, Projection, QuoteTarget};

use super::Measured;
use super::mcp::McpClient;

#[derive(Debug)]
pub(super) struct Line {
    pub node: String,
    pub hash: String,
    pub text: String,
}

pub(super) fn lines(payload: &Value) -> Vec<Line> {
    payload["lines"]
        .as_array()
        .expect("a read returns lines")
        .iter()
        .filter_map(Value::as_str)
        .filter_map(|line| {
            let (node, rest) = line.split_once(':')?;
            let (hash, text) = rest.split_once("| ")?;
            Some(Line {
                node: node.to_owned(),
                hash: hash.to_owned(),
                text: text.to_owned(),
            })
        })
        .collect()
}

pub(super) fn line<'l>(lines: &'l [Line], text: &str) -> &'l Line {
    lines
        .iter()
        .find(|line| line.text == text)
        .unwrap_or_else(|| panic!("no line reading {text:?}"))
}

pub(super) fn replace(line: &Line, hash: &str, text: &str) -> Value {
    op(&EditOp::ReplaceText {
        node: NodeTarget::Id(DocumentNodeId::new(&line.node)),
        expected_hash: Some(hash.to_owned()),
        text: text.to_owned(),
        content_as: ContentAs::Text,
    })
}

fn op(edit: &EditOp) -> Value {
    serde_json::to_value(edit).expect("an edit op serialises")
}

pub(super) fn markdown(text: &str) -> Value {
    op(&EditOp::AppendMarkdown {
        markdown: text.to_owned(),
    })
}

pub(super) fn texts(projection: &Projection) -> Vec<String> {
    projection
        .nodes()
        .filter(|node| node.children.is_empty())
        .filter_map(|node| node.text.clone())
        .filter(|text| !text.is_empty())
        .collect()
}

pub(super) async fn head(
    service: &DocumentService,
    who: &AuthenticatedUser,
    document: &DocumentId,
) -> Vec<String> {
    let projection = service
        .project(who, document, None, &TraceId::generate())
        .await
        .expect("read the head")
        .1;
    texts(&projection)
}

pub(super) async fn read(mcp: &mut McpClient, document: &DocumentId) -> (String, Vec<Line>) {
    let read = mcp
        .ok(
            "documents_read",
            json!({"document_id": document, "view": "text"}),
        )
        .await;
    let revision = read.payload["revision_id"]
        .as_str()
        .expect("a read names its revision")
        .to_owned();
    (revision, lines(&read.payload))
}

pub(super) async fn edit(
    mcp: &mut McpClient,
    document: &DocumentId,
    base: &str,
    ops: Vec<Value>,
) -> super::mcp::ToolResult {
    mcp.call(
        "documents_edit",
        json!({"document_id": document, "base_revision": base, "ops": ops}),
    )
    .await
}

pub(super) async fn create_memo(mcp: &mut McpClient, measured: &mut Measured) -> DocumentId {
    let created = mcp
        .ok(
            "documents_create",
            json!({
                "template_id": "dynapps-word",
                "name": format!("docs-smoke-{}", uuid::Uuid::new_v4().simple()),
                "params": {"title": "Smoke plan"},
                "ops": [markdown("## Memo\n\nalpha\n\nbeta\n\ngamma")],
            }),
        )
        .await;
    assert_eq!(created.payload["format"], "docx", "{}", created.raw);
    let document = DocumentId::new(
        created.payload["document_id"]
            .as_str()
            .expect("create names the document"),
    );
    let (_, shown) = read(mcp, &document).await;
    let texts: Vec<&str> = shown.iter().map(|l| l.text.as_str()).collect();
    for expected in ["# Smoke plan", "## Memo", "alpha", "beta", "gamma"] {
        assert!(texts.contains(&expected), "{expected} missing: {texts:?}");
    }
    measured.memo_in_one_create = true;
    document
}

pub(super) async fn guarded_edits(
    mcp: &mut McpClient,
    service: &DocumentService,
    owner: &AuthenticatedUser,
    document: &DocumentId,
    measured: &mut Measured,
) {
    let (base, shown) = read(mcp, document).await;
    let alpha = line(&shown, "alpha");
    let stale = edit(
        mcp,
        document,
        &base,
        vec![replace(alpha, "00000000", "nope")],
    )
    .await;
    assert!(stale.is_error, "a stale guard lands: {}", stale.raw);
    assert_eq!(stale.payload["code"], "guard_mismatch", "{}", stale.raw);

    let guarded = edit(
        mcp,
        document,
        &base,
        vec![replace(alpha, &alpha.hash, "alpha one")],
    )
    .await;
    assert!(!guarded.is_error, "{}", guarded.raw);
    measured.one_op_edit_bytes = guarded.model_bytes();
    measured.one_op_edit_wire_bytes = guarded.wire_bytes();

    let (base, shown) = read(mcp, document).await;
    let gamma = line(&shown, "gamma");
    let quoted = op(&EditOp::ReplaceText {
        node: NodeTarget::Find(QuoteTarget {
            find: "beta".to_owned(),
            nth: None,
            within: None,
        }),
        expected_hash: None,
        text: "beta two".to_owned(),
        content_as: ContentAs::Text,
    });
    let both = edit(
        mcp,
        document,
        &base,
        vec![quoted, replace(gamma, &gamma.hash, "gamma three")],
    )
    .await;
    assert!(!both.is_error, "{}", both.raw);
    assert_eq!(both.payload["applied"], 2, "{}", both.raw);
    measured.replace_and_next_in_one_edit = true;

    let (base, _) = read(mcp, document).await;
    let appended = edit(
        mcp,
        document,
        &base,
        vec![markdown("## Next steps\n\n- ship it")],
    )
    .await;
    assert!(!appended.is_error, "{}", appended.raw);

    let after = head(service, owner, document).await;
    for expected in [
        "alpha one",
        "beta two",
        "gamma three",
        "Next steps",
        "- ship it",
    ] {
        assert!(
            after.contains(&expected.to_owned()),
            "{expected} missing: {after:?}"
        );
    }

    let history = mcp
        .ok(
            "documents_history",
            json!({"document_id": document, "limit": 3}),
        )
        .await;
    let revisions = history.payload["revisions"]
        .as_array()
        .expect("history lists revisions");
    assert_eq!(revisions.len(), 3, "{}", history.raw);
    measured.history_bytes = history.model_bytes();
    measured.history_wire_bytes = history.wire_bytes();
}
