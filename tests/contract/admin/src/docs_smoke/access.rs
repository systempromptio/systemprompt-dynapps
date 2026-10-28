//! The people's half over REST: a group share and who may then read the
//! document, with a refusal carrying its code as `error_key`, the `/drive`
//! pages per viewer, and the bridge Drive feed through a merged and a
//! conflicting upload around agent edits.

use reqwest::StatusCode;
use serde_json::json;
use systemprompt::files::documents::DocumentService;
use systemprompt::files::documents::codecs::docx::DocxCodec;
use systemprompt::identifiers::{DocumentId, DocumentRevisionId, DriveDeviceId};
use systemprompt::models::auth::AuthenticatedUser;
use systemprompt::models::documents::{
    ContentAs, DriveCommitMeta, DriveCommitResponse, EditOp, NodeTarget, content_hash,
};
use systemprompt_provider_contracts::documents::DocumentCodec;

use super::flow::{edit, head, line, read, replace};
use super::mcp::McpClient;
use super::rest::Rest;

use crate::principal::CONTRACT_GROUP as GROUP;

#[derive(Clone, Copy)]
pub(super) struct Owner<'t> {
    pub(super) service: &'t DocumentService,
    pub(super) user: &'t AuthenticatedUser,
    pub(super) token: &'t str,
}

pub(super) struct Viewers<'t> {
    pub(super) owner: &'t str,
    pub(super) member: &'t str,
    pub(super) outsider: &'t str,
}

// A caller with no level is answered `not_found` and learns nothing more.
fn assert_not_found(reply: &super::rest::Reply, who: &str) {
    assert_eq!(reply.status, StatusCode::NOT_FOUND, "{who}: {}", reply.body);
    let envelope = reply.json();
    assert_eq!(envelope["error_key"], "not_found", "{who}: {}", reply.body);
    assert!(
        envelope["detail"].get("owner").is_none(),
        "{who} learns the owner: {}",
        reply.body
    );
}

pub(super) async fn share_to_group(rest: &Rest, viewers: &Viewers<'_>, document: &DocumentId) {
    let path = format!("/api/v1/documents/{}", document.as_str());
    let before = rest.get(&path, Some(viewers.member)).await;
    assert_not_found(&before, "a member before the share");
    let shared = rest
        .json(
            "POST",
            &format!("{path}/grants"),
            viewers.owner,
            &json!({"subject_kind": "group", "subject_value": GROUP, "level": "edit"}),
        )
        .await;
    assert!(
        shared.status.is_success(),
        "{}: {}",
        shared.status,
        shared.body
    );
    let after = rest.get(&path, Some(viewers.member)).await;
    assert_eq!(after.status, StatusCode::OK, "{}", after.body);
    let outsider = rest.get(&path, Some(viewers.outsider)).await;
    assert_not_found(&outsider, "an outsider");
}

pub(super) async fn drive_pages(rest: &Rest, viewers: &Viewers<'_>, document: &DocumentId) {
    let path = format!("/drive/{}", document.as_str());
    for (who, token) in [("owner", viewers.owner), ("member", viewers.member)] {
        let page = rest.get(&path, Some(token)).await;
        assert_eq!(page.status, StatusCode::OK, "the {who} opens the page");
        for element in ["<sp-comments", "<sp-proposal-list", "<sp-history"] {
            assert!(
                page.body.contains(element),
                "{element} missing for the {who}"
            );
        }
    }
    let outsider = rest.get(&path, Some(viewers.outsider)).await;
    assert_eq!(
        outsider.status,
        StatusCode::NOT_FOUND,
        "an outsider meets a 404"
    );
    let anonymous = rest.get(&path, None).await;
    assert!(
        anonymous.status.is_redirection(),
        "anonymous is sent to sign in: {}",
        anonymous.status
    );
    let listing = rest.get("/drive", Some(viewers.member)).await;
    assert_eq!(listing.status, StatusCode::OK);
    assert!(
        listing.body.contains("<sp-document-browser"),
        "the listing mounts the browser"
    );
    assert!(
        listing.body.contains("value=\"dynapps-word\""),
        "the dialog offers the install template"
    );
}

fn multipart(meta: &DriveCommitMeta, bytes: &[u8]) -> (String, Vec<u8>) {
    let boundary = format!("docs-smoke-{}", uuid::Uuid::new_v4().simple());
    let mut body = Vec::new();
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"meta\"\r\nContent-Type: application/json\r\n\r\n{}\r\n",
            serde_json::to_string(meta).expect("meta json")
        )
        .as_bytes(),
    );
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"blob\"; filename=\"doc.docx\"\r\nContent-Type: application/octet-stream\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    (format!("multipart/form-data; boundary={boundary}"), body)
}

fn edit_local(bytes: &[u8], from: &str, to: &str) -> Vec<u8> {
    let projection = DocxCodec.project(bytes).expect("project the local copy");
    let node = projection
        .nodes()
        .find(|n| n.children.is_empty() && n.text.as_deref() == Some(from))
        .expect("the node in the local copy")
        .clone();
    let op = EditOp::ReplaceText {
        node: NodeTarget::Id(node.id.clone()),
        expected_hash: Some(node.hash8),
        text: to.to_owned(),
        content_as: ContentAs::Text,
    };
    DocxCodec
        .apply(bytes, &projection, &[op])
        .expect("edit the local copy")
}

async fn upload(
    rest: &Rest,
    token: &str,
    document: &DocumentId,
    base: &str,
    bytes: &[u8],
) -> (StatusCode, String) {
    let meta = DriveCommitMeta {
        document_id: Some(document.clone()),
        path: "Smoke plan.docx".to_owned(),
        base_revision: Some(DocumentRevisionId::new(base)),
        content_sha256: content_hash(bytes),
        size_bytes: bytes.len() as u64,
        device_id: DriveDeviceId::new("docs-smoke-laptop"),
        message: None,
    };
    let (content_type, body) = multipart(&meta, bytes);
    let reply = rest
        .upload(
            "/v1/bridge/drive/commit",
            token,
            &[
                ("content-type", &content_type),
                ("if-match", &format!("\"{base}\"")),
            ],
            body,
        )
        .await;
    (reply.status, reply.body)
}

pub(super) async fn bridge_round_trip(
    rest: &Rest,
    mcp: &mut McpClient,
    owner: &Owner<'_>,
    document: &DocumentId,
) {
    let Owner {
        service,
        user: owner,
        token,
    } = *owner;
    let manifest = rest.get("/v1/bridge/drive/manifest", Some(token)).await;
    assert_eq!(manifest.status, StatusCode::OK, "{}", manifest.body);
    assert!(
        manifest.body.contains(document.as_str()),
        "the manifest lists the document"
    );

    let content = format!("/api/v1/documents/{}/content", document.as_str());
    let (base, shown) = read(mcp, document).await;
    let local = rest.bytes(&content, token).await;
    let alpha = line(&shown, "alpha one");
    let agent = edit(
        mcp,
        document,
        &base,
        vec![replace(alpha, &alpha.hash, "alpha by agent")],
    )
    .await;
    assert!(!agent.is_error, "{}", agent.raw);
    let (status, merged) = upload(
        rest,
        token,
        document,
        &base,
        &edit_local(&local, "gamma three", "gamma by person"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{merged}");
    let response: DriveCommitResponse = serde_json::from_str(&merged).expect("commit response");
    assert!(
        matches!(response, DriveCommitResponse::Merged { .. }),
        "{response:?}"
    );
    let shown_now = head(service, owner, document).await;
    assert!(
        shown_now.contains(&"alpha by agent".to_owned())
            && shown_now.contains(&"gamma by person".to_owned()),
        "{shown_now:?}"
    );

    let (base, shown) = read(mcp, document).await;
    let local = rest.bytes(&content, token).await;
    let target = line(&shown, "gamma by person");
    let agent = edit(
        mcp,
        document,
        &base,
        vec![replace(target, &target.hash, "gamma by agent")],
    )
    .await;
    assert!(!agent.is_error, "{}", agent.raw);
    let (status, conflict) = upload(
        rest,
        token,
        document,
        &base,
        &edit_local(&local, "gamma by person", "gamma by hand"),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{conflict}");
    let response: DriveCommitResponse = serde_json::from_str(&conflict).expect("commit response");
    assert!(
        matches!(&response, DriveCommitResponse::Conflict { base_revision_id: Some(named), .. } if named.as_str() == base),
        "{response:?}"
    );
    let shown_now = head(service, owner, document).await;
    assert!(
        shown_now.contains(&"gamma by agent".to_owned()),
        "the head edit wins: {shown_now:?}"
    );
}
