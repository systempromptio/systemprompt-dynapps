//! The shared document store end to end, as this install composes it.
//!
//! One throwaway database carries the full schema; the document service is
//! built the way the runtime builds it, over the install's templates and its
//! own group subject source, and the user-facing `/drive` pages and core's
//! bridge Drive feed are mounted beside the admin plane. The run creates a
//! document from the install's Word template, edits it through the service
//! as an agent (a stale node guard, then the right one, a quote target and a
//! Markdown insert), comments and resolves, shares it to a group, checks who
//! may open it and its page, turns an agent edit under a `propose` folder
//! into a proposal and accepts it, and drives the bridge feed through a merged
//! and a conflicting upload. A missing database fails the run.

use axum::http::StatusCode;
use systemprompt::files::documents::DocumentsError;
use systemprompt::files::documents::codecs::docx::DocxCodec;
use systemprompt::files::documents::models::{CommentAnchor, GrantTarget, RevisionAttribution};
use systemprompt::files::documents::revision::CommitBody;
use systemprompt::files::documents::service::{
    Author, CommentPolicy, CommentRequest, CommitOutcome, CommitRequest, CreateRequest,
    EditModeRequest, NewFolderRequest, ProposeRequest, ResolveCommentRequest, ReviewRequest,
    RevisionOrigin, ShareRequest,
};
use systemprompt::identifiers::{
    DocumentId, DocumentRevisionId, DocumentTemplateId, DriveDeviceId, McpExecutionId, TraceId,
    UserId,
};
use systemprompt::models::auth::{AuthenticatedUser, Permission};
use systemprompt::models::documents::{
    AccessLevel, ActorKind, AgentEditPolicy, ContentAs, DriveCommitMeta, DriveCommitResponse,
    EditMode, EditOp, NodeTarget, Projection, ProjectionNode, QuoteTarget, content_hash,
};
use systemprompt::runtime::documents::mcp::{refusal_code, tool_error};
use systemprompt_provider_contracts::documents::{DocumentCodec, TemplateParams};

use crate::app::{App, Call};
use crate::documents_fixture::{self, Store};
use crate::globals;
use crate::principal::{self, Credentials, Principal};

const GROUP: &str = "contract-group";

fn person(id: &UserId) -> AuthenticatedUser {
    AuthenticatedUser::new(
        id.clone(),
        id.as_str().to_owned(),
        format!("{}@contract.test", id.as_str()),
        vec![Permission::User],
    )
}

fn trace() -> TraceId {
    TraceId::generate()
}

fn agent() -> Author {
    Author {
        actor_kind: ActorKind::Agent,
        origin: RevisionOrigin::McpExecution(McpExecutionId::generate()),
        attribution: RevisionAttribution::default(),
    }
}

fn edit(document: &DocumentId, base: &DocumentRevisionId, ops: Vec<EditOp>) -> CommitRequest {
    CommitRequest {
        document_id: document.clone(),
        base_revision: Some(base.clone()),
        body: CommitBody::Ops(ops),
        author: agent(),
        message: Some("docs-smoke".to_owned()),
        lock_holder_id: None,
        comments: CommentPolicy::default(),
    }
}

fn landed(outcome: CommitOutcome) -> DocumentRevisionId {
    match outcome {
        CommitOutcome::Committed { revision, .. } | CommitOutcome::Merged { revision, .. } => {
            revision.id
        },
        CommitOutcome::Conflict { head, .. } => panic!("conflict at head {}", head.id),
    }
}

fn block<'p>(projection: &'p Projection, text: &str) -> &'p ProjectionNode {
    projection
        .nodes()
        .find(|node| node.children.is_empty() && node.text.as_deref() == Some(text))
        .unwrap_or_else(|| panic!("no block reading {text:?}"))
}

fn texts(projection: &Projection) -> Vec<String> {
    projection
        .nodes()
        .filter(|node| node.children.is_empty())
        .filter_map(|node| node.text.clone())
        .filter(|text| !text.is_empty())
        .collect()
}

fn replace(node: &ProjectionNode, hash: &str, text: &str) -> EditOp {
    EditOp::ReplaceText {
        node: NodeTarget::Id(node.id.clone()),
        expected_hash: Some(hash.to_owned()),
        text: text.to_owned(),
        content_as: ContentAs::Text,
    }
}

async fn head(store: &Store, who: &AuthenticatedUser, document: &DocumentId) -> Projection {
    store
        .service
        .project(who, document, None, &trace())
        .await
        .expect("read the head")
        .1
}

async fn create(store: &Store, owner: &AuthenticatedUser) -> (DocumentId, DocumentRevisionId) {
    let mut params = TemplateParams::default();
    params
        .values
        .insert("title".to_owned(), "Smoke plan".to_owned());
    let (document, revision) = store
        .service
        .create_from_template(
            owner,
            CreateRequest {
                template_id: DocumentTemplateId::new("dynapps-word"),
                folder: None,
                name: format!("docs-smoke-{}", uuid::Uuid::new_v4().simple()),
                params,
                ops: vec![EditOp::AppendMarkdown {
                    markdown: "alpha\n\nbeta\n\ngamma".to_owned(),
                }],
                author: agent(),
                message: Some("create".to_owned()),
            },
            &trace(),
        )
        .await
        .expect("create from dynapps-word");
    assert_eq!(document.format.as_str(), "docx");
    (document.id, revision.id)
}

async fn guarded_edits(
    store: &Store,
    owner: &AuthenticatedUser,
    document: &DocumentId,
    base: &DocumentRevisionId,
) -> DocumentRevisionId {
    let read = head(store, owner, document).await;
    let shown = texts(&read);
    assert!(shown.contains(&"Smoke plan".to_owned()), "{shown:?}");
    let alpha = block(&read, "alpha").clone();

    let stale = store
        .service
        .commit(
            owner,
            edit(document, base, vec![replace(&alpha, "00000000", "nope")]),
            &trace(),
        )
        .await
        .expect_err("a stale node guard refuses");
    let code = refusal_code(&tool_error(stale)).expect("a refusal code");
    assert_eq!(code.as_str(), "guard_mismatch");

    let guarded = edit(
        document,
        base,
        vec![replace(&alpha, &alpha.hash8, "alpha one")],
    );
    let second = landed(
        store
            .service
            .commit(owner, guarded, &trace())
            .await
            .expect("guarded edit"),
    );

    let quoted = EditOp::ReplaceText {
        node: NodeTarget::Find(QuoteTarget {
            find: "beta".to_owned(),
            nth: None,
            within: None,
        }),
        expected_hash: None,
        text: "beta two".to_owned(),
        content_as: ContentAs::Text,
    };
    let third = landed(
        store
            .service
            .commit(owner, edit(document, &second, vec![quoted]), &trace())
            .await
            .expect("quote-target edit"),
    );
    let inserted = EditOp::AppendMarkdown {
        markdown: "## Next steps\n\n- ship it".to_owned(),
    };
    let fourth = landed(
        store
            .service
            .commit(owner, edit(document, &third, vec![inserted]), &trace())
            .await
            .expect("markdown insert"),
    );
    let after = texts(&head(store, owner, document).await);
    for expected in ["alpha one", "beta two", "Next steps", "- ship it"] {
        assert!(
            after.contains(&expected.to_owned()),
            "{expected} missing: {after:?}"
        );
    }
    fourth
}

async fn comment_and_resolve(store: &Store, owner: &AuthenticatedUser, document: &DocumentId) {
    let read = head(store, owner, document).await;
    let node = block(&read, "gamma").id.clone();
    let comment = store
        .service
        .create_comment(
            owner,
            CommentRequest {
                document: document.clone(),
                anchor: Some(CommentAnchor {
                    node,
                    find: Some("gamma".to_owned()),
                    nth: None,
                }),
                body: "Check the figures".to_owned(),
                parent_id: None,
            },
            &trace(),
        )
        .await
        .expect("comment");
    let resolved = store
        .service
        .resolve_comment(
            owner,
            ResolveCommentRequest {
                document: document.clone(),
                comment: comment.id.clone(),
                answering: None,
            },
            &trace(),
        )
        .await
        .expect("resolve");
    assert_eq!(resolved.resolved_by.as_ref(), Some(&owner.id));
}

async fn share_to_group(
    store: &Store,
    owner: &AuthenticatedUser,
    member: &AuthenticatedUser,
    outsider: &AuthenticatedUser,
    document: &DocumentId,
) {
    let before = store.service.get(member, document, &trace()).await;
    assert!(
        matches!(
            before,
            Err(DocumentsError::Forbidden { .. } | DocumentsError::NotFound { .. })
        ),
        "a group member reads before the share: {before:?}"
    );
    store
        .service
        .share(
            owner,
            ShareRequest {
                target: GrantTarget::Document(document.clone()),
                subject_kind: "group".to_owned(),
                subject_value: GROUP.to_owned(),
                subject_display: None,
                level: AccessLevel::Edit,
                valid_until: None,
            },
            &trace(),
        )
        .await
        .expect("share to the group");
    let view = store
        .service
        .get(member, document, &trace())
        .await
        .expect("a group member reads after the share");
    assert_eq!(view.my_access, AccessLevel::Edit);
    let refused = store.service.get(outsider, document, &trace()).await;
    let code = refusal_code(&tool_error(refused.expect_err("an outsider is refused")))
        .expect("a refusal code");
    assert!(
        matches!(code.as_str(), "forbidden" | "not_found"),
        "{}",
        code.as_str()
    );
}

async fn drive_pages(app: &App, document: &DocumentId) {
    let path = format!("/drive/{}", document.as_str());
    for principal in [Principal::KnowledgeWorker, Principal::NonAdmin] {
        let (status, body) = app.call(Call::get(&path, principal)).await;
        assert_eq!(
            status,
            StatusCode::OK,
            "{} opens the page",
            principal.label()
        );
        for element in [
            "<sp-document-preview",
            "<sp-comments",
            "<sp-proposal-list",
            "<sp-history",
        ] {
            assert!(
                body.contains(element),
                "{element} missing for {}",
                principal.label()
            );
        }
    }
    let (status, _) = app.call(Call::get(&path, Principal::Admin)).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "an outsider meets a 404");
    let (status, _) = app.call(Call::get(&path, Principal::Anonymous)).await;
    assert!(
        status.is_redirection(),
        "anonymous is sent to sign in: {status}"
    );
    let (status, body) = app.call(Call::get("/drive", Principal::NonAdmin)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("<sp-document-browser"),
        "the listing mounts the browser"
    );
    assert!(
        body.contains("value=\"dynapps-word\""),
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

async fn upload(
    app: &App,
    token: &str,
    document: &DocumentId,
    base: &DocumentRevisionId,
    bytes: &[u8],
) -> (StatusCode, serde_json::Value) {
    let meta = DriveCommitMeta {
        document_id: Some(document.clone()),
        path: "Smoke plan.docx".to_owned(),
        base_revision: Some(base.clone()),
        content_sha256: content_hash(bytes),
        size_bytes: bytes.len() as u64,
        device_id: DriveDeviceId::new("docs-smoke-laptop"),
        message: None,
    };
    let (content_type, body) = multipart(&meta, bytes);
    let (status, text) = app
        .send_bytes(
            "/v1/bridge/drive/commit",
            token,
            &[
                ("content-type", &content_type),
                ("if-match", &format!("\"{}\"", base.as_str())),
            ],
            body,
        )
        .await;
    let json = serde_json::from_str(&text).unwrap_or(serde_json::Value::String(text));
    (status, json)
}

fn upload_over(bytes: &[u8], node: &ProjectionNode, text: &str) -> Vec<u8> {
    let paths = DocxCodec.project(bytes).expect("project the local copy");
    let path = paths
        .nodes()
        .find(|n| n.children.is_empty() && n.text == node.text)
        .expect("the node in the local copy")
        .clone();
    DocxCodec
        .apply(bytes, &paths, &[replace(&path, &path.hash8, text)])
        .expect("edit the local copy")
}

async fn bridge_round_trip(
    app: &App,
    store: &Store,
    owner: &AuthenticatedUser,
    token: &str,
    document: &DocumentId,
) {
    let (status, manifest) = app
        .call_with_bearer_get("/v1/bridge/drive/manifest", token)
        .await;
    assert_eq!(status, StatusCode::OK, "{manifest}");
    assert!(
        manifest.contains(document.as_str()),
        "the manifest lists the document"
    );

    let (base, local) = store
        .service
        .read_bytes(owner, document, None, &trace())
        .await
        .expect("the bridge's local copy");
    let read = head(store, owner, document).await;
    let gamma = block(&read, "gamma").clone();
    let alpha = block(&read, "alpha one").clone();
    landed(
        store
            .service
            .commit(
                owner,
                edit(
                    document,
                    &base.id,
                    vec![replace(&alpha, &alpha.hash8, "alpha by agent")],
                ),
                &trace(),
            )
            .await
            .expect("an MCP edit on another node"),
    );
    let (status, merged) = upload(
        app,
        token,
        document,
        &base.id,
        &upload_over(&local, &gamma, "gamma by person"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{merged}");
    let response: DriveCommitResponse = serde_json::from_value(merged).expect("commit response");
    assert!(
        matches!(response, DriveCommitResponse::Merged { .. }),
        "{response:?}"
    );
    let shown = texts(&head(store, owner, document).await);
    assert!(
        shown.contains(&"alpha by agent".to_owned())
            && shown.contains(&"gamma by person".to_owned()),
        "{shown:?}"
    );

    let (base, local) = store
        .service
        .read_bytes(owner, document, None, &trace())
        .await
        .expect("a fresh local copy");
    let read = head(store, owner, document).await;
    let target = block(&read, "gamma by person").clone();
    landed(
        store
            .service
            .commit(
                owner,
                edit(
                    document,
                    &base.id,
                    vec![replace(&target, &target.hash8, "gamma by agent")],
                ),
                &trace(),
            )
            .await
            .expect("an MCP edit on the same node"),
    );
    let (status, conflict) = upload(
        app,
        token,
        document,
        &base.id,
        &upload_over(&local, &target, "gamma by hand"),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{conflict}");
    let shown = texts(&head(store, owner, document).await);
    assert!(
        shown.contains(&"gamma by agent".to_owned()),
        "the head edit wins: {shown:?}"
    );
}

async fn propose_and_accept(store: &Store, owner: &AuthenticatedUser) {
    let folder = store
        .service
        .create_folder(
            owner,
            NewFolderRequest {
                parent: None,
                name: format!("review-{}", uuid::Uuid::new_v4().simple()),
            },
            &trace(),
        )
        .await
        .expect("create a folder");
    store
        .service
        .set_folder_agent_edit_policy(owner, &folder.id, AgentEditPolicy::Propose, &trace())
        .await
        .expect("agents propose in this folder");
    let (document, revision) = store
        .service
        .create_from_template(
            owner,
            CreateRequest {
                template_id: DocumentTemplateId::new("dynapps-word"),
                folder: Some(folder.id.clone()),
                name: "Reviewed plan".to_owned(),
                params: TemplateParams::default(),
                ops: vec![EditOp::AppendMarkdown {
                    markdown: "draft".to_owned(),
                }],
                author: agent(),
                message: None,
            },
            &trace(),
        )
        .await
        .expect("create under the propose folder");
    let mode = store
        .service
        .edit_mode(
            owner,
            EditModeRequest {
                document: &document.id,
                requested: None,
                actor: ActorKind::Agent,
            },
            &trace(),
        )
        .await
        .expect("edit mode");
    assert_eq!(
        mode,
        EditMode::Propose,
        "an agent proposes under the folder"
    );
    let draft = block(&head(store, owner, &document.id).await, "draft").clone();
    let proposed = store
        .service
        .propose(
            owner,
            ProposeRequest {
                document_id: document.id.clone(),
                base_revision: Some(revision.id.clone()),
                ops: vec![replace(&draft, &draft.hash8, "final")],
                author: agent(),
                message: Some("tighten".to_owned()),
            },
            &trace(),
        )
        .await
        .expect("propose");
    let unchanged = texts(&head(store, owner, &document.id).await);
    assert!(
        unchanged.contains(&"draft".to_owned()),
        "a proposal leaves the head: {unchanged:?}"
    );
    store
        .service
        .accept_proposal(
            owner,
            ReviewRequest {
                proposal_id: proposed.proposal.id.clone(),
                note: Some("ship".to_owned()),
                reviewer: Author {
                    actor_kind: ActorKind::User,
                    origin: RevisionOrigin::Api,
                    attribution: RevisionAttribution::default(),
                },
            },
            &trace(),
        )
        .await
        .expect("accept");
    let accepted = texts(&head(store, owner, &document.id).await);
    assert!(
        accepted.contains(&"final".to_owned()),
        "accepting lands it: {accepted:?}"
    );
}

// The bridge feed checks that the bearer's session is a live row, which the
// contract principals' browser tokens never needed.
async fn bridge_token(pool: &sqlx::PgPool, user: &UserId) -> String {
    let session_id = systemprompt::identifiers::SessionId::new(uuid::Uuid::new_v4().to_string());
    sqlx::query("INSERT INTO user_sessions (session_id, user_id) VALUES ($1, $2)")
        .bind(session_id.as_str())
        .bind(user.as_str())
        .execute(pool)
        .await
        .expect("seed the bridge session");
    systemprompt_security::JwtService::generate_admin_token(
        globals::signing_keys(),
        &systemprompt_security::AdminTokenParams {
            user_id: user,
            session_id: &session_id,
            email: &format!("{}@contract.test", user.as_str()),
            issuer: &globals::jwt_issuer(),
            duration: chrono::Duration::hours(1),
            client_id: None,
        },
    )
    .expect("mint a bridge token")
    .as_str()
    .to_owned()
}

#[tokio::test(flavor = "multi_thread")]
async fn docs_smoke_the_document_store_end_to_end() {
    assert!(globals::init(), "the contract fixture profile boots");
    let db = crate::tempdb::TempDb::create()
        .await
        .expect("docs-smoke needs SYSTEMPROMPT_TEST_DATABASE_URL");
    let credentials: Credentials = principal::provision(&db.pool).await;
    let owner = person(&credentials.knowledge_worker_user_id);
    let member = person(&credentials.non_admin_user_id);
    let outsider = person(&credentials.admin_user_id);
    let owner_token = bridge_token(&db.pool, &credentials.knowledge_worker_user_id).await;
    let store = documents_fixture::store(&db.pool);
    let app = App::with_documents(
        &db.pool,
        credentials,
        documents_fixture::mount(&db.pool, &store),
    );

    let (document, first) = create(&store, &owner).await;
    guarded_edits(&store, &owner, &document, &first).await;
    comment_and_resolve(&store, &owner, &document).await;
    share_to_group(&store, &owner, &member, &outsider, &document).await;
    propose_and_accept(&store, &owner).await;
    drive_pages(&app, &document).await;
    bridge_round_trip(&app, &store, &owner, &owner_token, &document).await;

    db.cleanup().await;
}
