//! A live page as it is drawn: the stored HTML with the page runtime at the
//! top of its head (after the doctype, or first, when it has no head), so the
//! runtime runs before any script of the page, and the policy it runs under.
//!
//! [`framed_page`] builds it for both who draw it: the viewer shell's frame
//! (`/drive/{id}/live/frame`, which sends the policy as its own header) and
//! a native client, which asks `GET /api/v1/artifacts/{document_id}/page
//! [?revision=]` (`read`) with the person's session, as every broker route
//! takes it. That answer is the page as `text/html`, the policy in
//! `Content-Security-Policy` and the revision it carries in `X-Revision`; the
//! client sets the policy on the sandboxed document it draws the page in.
//! Who may read it is who may read the frame: a person who sees the
//! document, which is an HTML document, at a revision that is one of its own
//! (`not_found` otherwise).
//!
//! The runtime (`runtime-boot.js`, `runtime-core.js`, `runtime-mcp.js` and
//! `runtime-use.js` under `storage/files/js/workspace/live/`, compiled in
//! and joined into one script, each a complete function expression) is the
//! page's only way out: `window.governance.use`, and its alias
//! `window.claude`, post to the page's parent. It reads answers only from
//! that parent and, when it is given origins, only from those: the frame's
//! runtime is given this site's, while a native client's page is given none,
//! because its parent is the client itself, never this site. The policy
//! keeps the page there: `sandbox allow-scripts` without
//! `allow-same-origin` makes it an opaque-origin document that holds no
//! session and reads no cookie; `connect-src 'none'` and `form-action 'none'`
//! give it no network. Whether the page's own scripts run, and which origins
//! it loads styles, fonts and scripts from, is the instance's rendered-HTML
//! tier for the document (`files.documents.renderedHtml`), as on
//! `/rendered`; with the tier's scripts off only the runtime runs, by its
//! nonce, and the page stays static.

use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::header::{
    CACHE_CONTROL, CONTENT_SECURITY_POLICY, CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS,
};
use axum::http::{HeaderMap, HeaderName, HeaderValue};
use axum::response::Response;
use serde::Deserialize;
use systemprompt::files::RemoteOrigin;
use systemprompt::files::documents::agents::{DocumentActor, act_as};
use systemprompt::files::documents::{DocumentService, DocumentsError};
use systemprompt::identifiers::{DocumentId, DocumentRevisionId, TraceId};
use systemprompt::models::auth::AuthenticatedUser;
use systemprompt::models::documents::DocumentFormat;

use super::ArtifactsBroker;
use super::error::{ArtifactCallCode, ArtifactCallError};
use super::manifest::CONTRACT_VERSION;
use super::viewer::ArtifactViewer;

const RUNTIME: [&str; 4] = [
    include_str!("../../../../../../storage/files/js/workspace/live/runtime-boot.js"),
    include_str!("../../../../../../storage/files/js/workspace/live/runtime-core.js"),
    include_str!("../../../../../../storage/files/js/workspace/live/runtime-mcp.js"),
    include_str!("../../../../../../storage/files/js/workspace/live/runtime-use.js"),
];
const REVISION_HEADER: &str = "x-revision";

pub(crate) struct FramedPage {
    pub(crate) html: Vec<u8>,
    pub(crate) policy: String,
    pub(crate) revision: DocumentRevisionId,
}

#[derive(Debug, Default, Deserialize)]
pub(super) struct PageQuery {
    #[serde(default)]
    revision: Option<String>,
}

pub(super) async fn read(
    State(broker): State<ArtifactsBroker>,
    Path(raw_document): Path<String>,
    Query(query): Query<PageQuery>,
    headers: HeaderMap,
) -> Result<Response, ArtifactCallError> {
    let viewer = ArtifactViewer::from_request(&broker.inner.parts.boot, &headers)?;
    let document = DocumentId::try_new(raw_document).map_err(|_invalid| not_found())?;
    let revision = query
        .revision
        .filter(|raw| !raw.is_empty())
        .map(DocumentRevisionId::try_new)
        .transpose()
        .map_err(|_invalid| not_found())?;
    act_as(DocumentActor::Human, async {
        let page = broker.page(&viewer, &document).await?;
        if page.view.document.format != DocumentFormat::Html {
            return Err(not_found());
        }
        let framed = framed_page(
            &broker.inner.parts.documents.0,
            &viewer.user,
            &document,
            revision.as_ref(),
            &[],
        )
        .await
        .map_err(|error| match error {
            DocumentsError::Forbidden { .. }
            | DocumentsError::NotFound { .. }
            | DocumentsError::Trashed { .. } => not_found(),
            other => {
                tracing::error!(error = %other, "the document service failed a live page read");
                unreadable()
            },
        })?;
        answered(framed)
    })
    .await
}

pub(crate) async fn framed_page(
    service: &DocumentService,
    who: &AuthenticatedUser,
    document: &DocumentId,
    revision: Option<&DocumentRevisionId>,
    origins: &[String],
) -> Result<FramedPage, DocumentsError> {
    let (stored_revision, stored) = service
        .read_bytes(who, document, revision, &TraceId::generate())
        .await?;
    let agent_content = service.agent_content(document).await?;
    let tier = service.config().rendered_html.tier(agent_content);
    let listed: Vec<&str> = tier
        .remote_origins
        .iter()
        .map(RemoteOrigin::as_str)
        .collect();
    let nonce = uuid::Uuid::new_v4().simple().to_string();
    Ok(FramedPage {
        html: with_runtime(&stored, &runtime_tag(&nonce, origins)),
        policy: policy(tier.scripts, &listed, &nonce),
        revision: stored_revision.id,
    })
}

fn answered(framed: FramedPage) -> Result<Response, ArtifactCallError> {
    let csp = HeaderValue::from_str(&framed.policy).map_err(|_invalid| unreadable())?;
    let revision =
        HeaderValue::from_str(framed.revision.as_str()).map_err(|_invalid| unreadable())?;
    let mut response = Response::new(Body::from(framed.html));
    let headers = response.headers_mut();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(CONTENT_SECURITY_POLICY, csp);
    headers.insert(HeaderName::from_static(REVISION_HEADER), revision);
    Ok(response)
}

const fn not_found() -> ArtifactCallError {
    ArtifactCallError::new(ArtifactCallCode::NotFound, "document not found")
}

const fn unreadable() -> ArtifactCallError {
    ArtifactCallError::new(ArtifactCallCode::Internal, "the page could not be read")
}

fn runtime_tag(nonce: &str, origins: &[String]) -> String {
    let host = serde_json::json!({ "origins": origins, "contract": CONTRACT_VERSION })
        .to_string()
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026");
    let [boot, core, mcp, usage] = RUNTIME;
    format!("<script nonce=\"{nonce}\">({boot})({core}, {mcp}, {usage}, {host})</script>")
}

fn policy(scripts: bool, origins: &[&str], nonce: &str) -> String {
    let listed: String = origins.iter().flat_map(|origin| [" ", origin]).collect();
    let script_src = if scripts {
        format!("'unsafe-inline' 'unsafe-eval'{listed}")
    } else {
        format!("'nonce-{nonce}'")
    };
    format!(
        "sandbox allow-scripts; default-src 'none'; script-src {script_src}; style-src \
         'unsafe-inline'{listed}; img-src data: blob:{listed}; font-src data:{listed}; media-src \
         data: blob:{listed}; connect-src 'none'; form-action 'none'; base-uri 'none'"
    )
}

fn with_runtime(page: &[u8], runtime: &str) -> Vec<u8> {
    let at = tag_end(page, b"<head")
        .or_else(|| tag_end(page, b"<!doctype"))
        .unwrap_or(0);
    let mut out = Vec::with_capacity(page.len() + runtime.len());
    out.extend_from_slice(&page[..at]);
    out.extend_from_slice(runtime.as_bytes());
    out.extend_from_slice(&page[at..]);
    out
}

fn tag_end(page: &[u8], tag: &[u8]) -> Option<usize> {
    let start = page.windows(tag.len() + 1).position(|window| {
        window[..tag.len()].eq_ignore_ascii_case(tag)
            && matches!(window[tag.len()], b'>' | b' ' | b'\t' | b'\n' | b'\r')
    })?;
    page[start..]
        .iter()
        .position(|byte| *byte == b'>')
        .map(|offset| start + offset + 1)
}
