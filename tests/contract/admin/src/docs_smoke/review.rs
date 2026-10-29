//! The review half of the agent's run over the binary: a comment created
//! and resolved, and an edit under a `propose` folder that becomes a
//! proposal, leaves the head alone and lands when accepted. The folder and
//! its policy are set over REST, as an administrator sets them.

use serde_json::json;
use systemprompt::files::documents::DocumentService;
use systemprompt::identifiers::DocumentId;
use systemprompt::models::auth::AuthenticatedUser;

use super::flow::{edit, head, line, markdown, read, replace};
use super::mcp::McpClient;
use super::rest::Rest;

use crate::principal::CONTRACT_GROUP as GROUP;

pub(super) async fn comment_and_resolve(mcp: &mut McpClient, document: &DocumentId) {
    let (_, shown) = read(mcp, document).await;
    let gamma = line(&shown, "gamma three");
    let created = mcp
        .ok(
            "documents_comment",
            json!({
                "action": "create",
                "document_id": document,
                "node": gamma.node,
                "anchor": {"find": "gamma"},
                "body": "Check the figures",
            }),
        )
        .await;
    let comment = created.payload["comments"][0]["id"]
        .as_str()
        .expect("the comment's id")
        .to_owned();
    let resolved = mcp
        .ok(
            "documents_comment",
            json!({"action": "resolve", "document_id": document, "comment_id": comment}),
        )
        .await;
    assert!(
        resolved.payload["comments"][0]["resolved_by"].is_string(),
        "{}",
        resolved.raw
    );
}

pub(super) async fn propose_and_accept(
    mcp: &mut McpClient,
    rest: &Rest,
    token: &str,
    reviewer: &str,
    service: &DocumentService,
    owner: &AuthenticatedUser,
) {
    let folder = rest
        .json(
            "POST",
            "/api/v1/document-folders",
            token,
            &json!({"name": format!("review-{}", uuid::Uuid::new_v4().simple())}),
        )
        .await;
    assert!(
        folder.status.is_success(),
        "{}: {}",
        folder.status,
        folder.body
    );
    let folder = folder.json()["id"]
        .as_str()
        .expect("the folder's id")
        .to_owned();
    let policy = rest
        .json(
            "PATCH",
            &format!("/api/v1/document-folders/{folder}"),
            token,
            &json!({"agent_edit_policy": "propose"}),
        )
        .await;
    assert!(
        policy.status.is_success(),
        "{}: {}",
        policy.status,
        policy.body
    );

    let created = mcp
        .ok(
            "documents_create",
            json!({
                "template_id": "dynapps-word",
                "folder_id": folder,
                "name": "Reviewed plan",
                "ops": [markdown("draft")],
            }),
        )
        .await;
    let document = DocumentId::new(
        created.payload["document_id"]
            .as_str()
            .expect("create names the document"),
    );
    let (base, shown) = read(mcp, &document).await;
    let draft = line(&shown, "draft");
    let proposed = edit(
        mcp,
        &document,
        &base,
        vec![replace(draft, &draft.hash, "final")],
    )
    .await;
    assert!(!proposed.is_error, "{}", proposed.raw);
    let proposal = proposed.payload["proposal_id"]
        .as_str()
        .unwrap_or_else(|| panic!("an agent edit under the folder proposes: {}", proposed.raw))
        .to_owned();
    let unchanged = head(service, owner, &document).await;
    assert!(
        unchanged.contains(&"draft".to_owned()),
        "a proposal leaves the head: {unchanged:?}"
    );
    // Core refuses an agent's review and a proposer reviewing their own
    // proposal, so a second editor from the group accepts over REST.
    let shared = rest
        .json(
            "POST",
            &format!("/api/v1/documents/{}/grants", document.as_str()),
            token,
            &json!({"subject_kind": "group", "subject_value": GROUP, "level": "edit"}),
        )
        .await;
    assert!(
        shared.status.is_success(),
        "{}: {}",
        shared.status,
        shared.body
    );
    let accept = rest
        .json(
            "POST",
            &format!(
                "/api/v1/documents/{}/proposals/{proposal}/accept",
                document.as_str()
            ),
            reviewer,
            &json!({"note": "ship"}),
        )
        .await;
    assert!(
        accept.status.is_success(),
        "{}: {}",
        accept.status,
        accept.body
    );
    let accepted = head(service, owner, &document).await;
    assert!(
        accepted.contains(&"final".to_owned()),
        "accepting lands it: {accepted:?}"
    );
}
