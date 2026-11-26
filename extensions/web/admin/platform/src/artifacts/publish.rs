//! `POST /api/v1/artifacts/{document_id}/artifact/publish`: a page writes
//! itself as a new version of its document, `{html, base_revision}`.
//!
//! The page must declare `artifact` (without `publish: false`;
//! `not_declared` otherwise), the viewer must be able to edit the document
//! (`not_writer`), and the HTML must be a whole page, starting with its
//! doctype (`invalid_content`), of at most `MAX_PAGE_BYTES` (`too_large`).
//! The write is compare-and-set on the version the viewer loaded: a base that
//! is not the head is a `conflict` naming the live version. It lands through
//! core's document service as the viewer, byte for byte, so the page keeps
//! its scripts; comments anchored in the old page detach rather than block
//! it. The manifest stays with the document.

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::json;
use systemprompt::files::documents::DocumentsError;
use systemprompt::files::documents::agents::{DocumentActor, act_as};
use systemprompt::files::documents::model::Media;
use systemprompt::files::documents::models::{RevisionAttribution, RevisionOrigin};
use systemprompt::files::documents::revision::CommitBody;
use systemprompt::files::documents::service::{
    Author, CommentPolicy, CommitOutcome, CommitRequest,
};
use systemprompt::identifiers::{DocumentId, DocumentRevisionId, TraceId};
use systemprompt::models::documents::{AccessLevel, ActorKind, DocumentFormat};

use super::ArtifactsBroker;
use super::admit::Page;
use super::error::{ArtifactCallCode, ArtifactCallError};
use super::viewer::ArtifactViewer;

const MAX_PAGE_BYTES: usize = 2 * 1024 * 1024;
const DOCTYPE: &str = "<!doctype html";
const MESSAGE: &str = "Published from the page";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PublishBody {
    html: String,
    base_revision: String,
}

pub(super) async fn publish(
    State(broker): State<ArtifactsBroker>,
    Path(raw_document): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, ArtifactCallError> {
    let viewer = ArtifactViewer::from_request(&broker.inner.parts.boot, &headers)?;
    let document = DocumentId::try_new(raw_document).map_err(|_invalid| {
        ArtifactCallError::new(ArtifactCallCode::NotFound, "document not found")
    })?;
    let body: PublishBody = serde_json::from_slice(&body).map_err(|error| {
        tracing::debug!(%error, "a page publish body did not parse");
        ArtifactCallError::new(
            ArtifactCallCode::BadRequest,
            "the body is `{html, base_revision}`",
        )
    })?;
    Box::pin(act_as(
        DocumentActor::Human,
        write(&broker, &viewer, &document, body),
    ))
    .await
}

async fn write(
    broker: &ArtifactsBroker,
    viewer: &ArtifactViewer,
    document: &DocumentId,
    body: PublishBody,
) -> Result<Response, ArtifactCallError> {
    let page = broker.page(viewer, document).await?;
    let head = admitted(&page, &body)?;
    let request = CommitRequest {
        document_id: document.clone(),
        base_revision: Some(head),
        body: CommitBody::Bytes(body.html.into_bytes()),
        author: Author {
            actor_kind: ActorKind::User,
            origin: RevisionOrigin::Api,
            attribution: RevisionAttribution::default(),
        },
        message: Some(MESSAGE.to_owned()),
        lock_holder: None,
        comments: CommentPolicy::detach(),
        op_id: None,
        media: Media::new(),
    };
    let committed = broker
        .inner
        .parts
        .documents
        .0
        .commit(&viewer.user, request, &TraceId::generate())
        .await;
    answer(committed)
}

fn admitted(page: &Page, body: &PublishBody) -> Result<DocumentRevisionId, ArtifactCallError> {
    let declared = page
        .manifest
        .as_ref()
        .is_some_and(super::manifest::Capabilities::publishes);
    if !declared || page.view.document.format != DocumentFormat::Html {
        return Err(ArtifactCallError::new(
            ArtifactCallCode::NotDeclared,
            "this page does not declare `artifact`",
        ));
    }
    if page.view.my_access < AccessLevel::Edit {
        return Err(ArtifactCallError::new(
            ArtifactCallCode::NotWriter,
            "the viewer cannot edit this page",
        ));
    }
    if body.html.len() > MAX_PAGE_BYTES {
        return Err(ArtifactCallError::new(
            ArtifactCallCode::TooLarge,
            "the page is larger than a publish may be",
        ));
    }
    if !is_whole_page(&body.html) {
        return Err(ArtifactCallError::new(
            ArtifactCallCode::InvalidContent,
            "a publish is a whole page that starts with its doctype",
        ));
    }
    let Some(head) = page.view.document.head_revision_id.clone() else {
        return Err(ArtifactCallError::new(
            ArtifactCallCode::NotDeclared,
            "this page has no version to replace",
        ));
    };
    if body.base_revision != head.as_str() {
        return Err(ArtifactCallError::conflict(head.as_str().to_owned()));
    }
    Ok(head)
}

fn answer(committed: Result<CommitOutcome, DocumentsError>) -> Result<Response, ArtifactCallError> {
    match committed {
        Ok(CommitOutcome::Committed(landed) | CommitOutcome::Merged(landed)) => {
            Ok(Json(json!({ "version": landed.revision.id })).into_response())
        },
        Ok(CommitOutcome::Conflict { head, .. }) => {
            Err(ArtifactCallError::conflict(head.id.as_str().to_owned()))
        },
        Err(DocumentsError::Forbidden { .. }) => Err(ArtifactCallError::new(
            ArtifactCallCode::NotWriter,
            "the viewer cannot edit this page",
        )),
        Err(DocumentsError::TooLarge { .. }) => Err(ArtifactCallError::new(
            ArtifactCallCode::TooLarge,
            "the page is larger than this document may be",
        )),
        Err(
            error @ (DocumentsError::ContentRefused { .. } | DocumentsError::InvalidParams { .. }),
        ) => Err(ArtifactCallError::told(
            ArtifactCallCode::InvalidContent,
            error.to_string(),
        )),
        Err(error) => {
            tracing::error!(%error, "the document service refused a page's publish");
            Err(ArtifactCallError::new(
                ArtifactCallCode::Internal,
                "the page could not be published",
            ))
        },
    }
}

fn is_whole_page(html: &str) -> bool {
    html.trim_start_matches('\u{feff}')
        .trim_start()
        .get(..DOCTYPE.len())
        .is_some_and(|start| start.eq_ignore_ascii_case(DOCTYPE))
}
