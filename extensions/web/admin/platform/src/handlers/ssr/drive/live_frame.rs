//! `/drive/{document_id}/live/frame`: a live page as its sandboxed frame
//! loads it, the stored HTML with the page runtime at the top of its head
//! (after the doctype, or first, when it has no head), so the runtime runs
//! before any script of the page.
//!
//! The page and its policy are built by the broker's `framed_page`, which
//! also serves them to a native client: the runtime (the page's only way out,
//! posting to the frame's parent, the viewer shell) is given this site's
//! origins here. The response's policy keeps the page there: `sandbox
//! allow-scripts` without `allow-same-origin` makes it an opaque-origin
//! document that holds no session and reads no cookie, framed or opened on
//! its own; `connect-src 'none'` and `form-action 'none'` give it no
//! network; and the frame options let only this site frame it. Whether the
//! page's own scripts run, and which origins it loads styles, fonts and
//! scripts from, is the instance's rendered-HTML tier for the document
//! (`files.documents.renderedHtml`), as on `/rendered`; with the tier's
//! scripts off only the runtime runs, by its nonce, and the page stays
//! static. `?revision=` serves the version the shell loaded, so a publish
//! compares against what the viewer saw.

use axum::Extension;
use axum::body::Body;
use axum::extract::{Path, Query};
use axum::http::{HeaderValue, header};
use axum::response::Response;
use serde::Deserialize;
use systemprompt::extension::FrameOptionsOverride;
use systemprompt::files::documents::DocumentsError;
use systemprompt::identifiers::DocumentRevisionId;
use systemprompt::manifest::profile::FrameOptions;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;
use systemprompt_web_admin_core::ssr::page::Page;

use super::live::live_document;
use super::user::{DriveDocuments, caller};
use crate::artifacts::framed_page;
use crate::routes::drive_actor::site_origins;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct FrameQuery {
    #[serde(default)]
    revision: Option<String>,
}

pub(crate) async fn drive_live_frame(
    shell: Page,
    Extension(documents): Extension<DriveDocuments>,
    Extension(boot): Extension<ExtensionBoot>,
    Path(raw_id): Path<String>,
    Query(query): Query<FrameQuery>,
) -> AdminHtmlResult<Response> {
    let not_found = || AdminError::NotFound("Document not found.".to_owned());
    let (document_id, _) = live_document(&shell, &documents, raw_id).await?;
    let revision = query
        .revision
        .filter(|raw| !raw.is_empty())
        .map(DocumentRevisionId::try_new)
        .transpose()
        .map_err(|_invalid| not_found())?;
    let page = framed_page(
        &documents.0,
        &caller(&shell.user),
        &document_id,
        revision.as_ref(),
        &site_origins(&boot),
    )
    .await
    .map_err(|error| match error {
        DocumentsError::Forbidden { .. }
        | DocumentsError::NotFound { .. }
        | DocumentsError::Trashed { .. } => not_found(),
        other => AdminError::internal(other),
    })?;
    let csp = HeaderValue::from_str(&page.policy).map_err(AdminError::internal)?;
    Ok(framed(page.html, csp))
}

fn framed(body: Vec<u8>, csp: HeaderValue) -> Response {
    let mut response = Response::new(Body::from(body));
    response
        .extensions_mut()
        .insert(FrameOptionsOverride(FrameOptions::SameOrigin));
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(header::CONTENT_SECURITY_POLICY, csp);
    response
}
