//! `/drive/{document_id}/live`: the viewer shell of a live page, for the
//! person signed in to the web.
//!
//! The page carries only the document's paths; its script
//! (`js/workspace/live/`) reads the viewer's permissions from the artifacts
//! broker, frames the page from `/live/frame` in an opaque-origin sandbox and
//! answers the page's runtime through the broker, drawing the viewer's prompts
//! itself. With `?embed=1` the bar is left out, so a client that shows the
//! shell (`DynWork` desktop, Mobile) shows only the page, the prompts and the
//! status line. Without a web session the Drive's gate sends the request to the
//! web login with this path, query included, as the way back. The shell keeps
//! the instance's framing policy: a client shows it as a page of its own, never
//! inside another site's frame.

use axum::Extension;
use axum::extract::{Path, Query};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use systemprompt::files::documents::DocumentsError;
use systemprompt::identifiers::{DocumentId, TraceId};
use systemprompt::models::ApiPaths;
use systemprompt::models::documents::DocumentFormat;

use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_workspace_page};
use systemprompt_web_admin_core::ssr::page::Page;

use super::user::{DriveDocuments, USER_BASE_URL, caller, ensure_enabled};

const SIGN_IN: &str = "/admin/login";
const CONNECTORS: &str = "/admin/connectors";
const EMBED: &str = "1";

#[derive(Debug, Default, Deserialize)]
pub(crate) struct LiveQuery {
    #[serde(default)]
    embed: Option<String>,
}

#[derive(Debug, Serialize)]
struct DriveLivePage {
    page: &'static str,
    title: String,
    embed: bool,
    document_id: DocumentId,
    document_url: String,
    frame_url: String,
    api: String,
    people_api: String,
    search_api: String,
    sign_in: String,
    connectors: &'static str,
}

pub(super) async fn live_document(
    shell: &Page,
    documents: &DriveDocuments,
    raw_id: String,
) -> Result<(DocumentId, String), AdminError> {
    let not_found = || AdminError::NotFound("Document not found.".to_owned());
    ensure_enabled(shell)?;
    let document_id = DocumentId::try_new(raw_id).map_err(|_invalid| not_found())?;
    let view = documents
        .0
        .get(&caller(&shell.user), &document_id, &TraceId::generate())
        .await
        .map_err(|error| match error {
            DocumentsError::Forbidden { .. }
            | DocumentsError::NotFound { .. }
            | DocumentsError::Trashed { .. } => not_found(),
            other => AdminError::internal(other),
        })?;
    if view.document.format != DocumentFormat::Html {
        return Err(not_found());
    }
    Ok((document_id, view.document.name))
}

pub(crate) async fn drive_live_page(
    shell: Page,
    Extension(documents): Extension<DriveDocuments>,
    Path(raw_id): Path<String>,
    Query(query): Query<LiveQuery>,
) -> AdminHtmlResult<Response> {
    let (document_id, title) = live_document(&shell, &documents, raw_id).await?;
    let embed = query.embed.as_deref() == Some(EMBED);
    let encoded = urlencoding::encode(document_id.as_str()).into_owned();
    let document_url = format!("{USER_BASE_URL}/{encoded}");
    let back = format!("{document_url}/live{}", if embed { "?embed=1" } else { "" });
    let page = DriveLivePage {
        page: "drive-live",
        title,
        embed,
        frame_url: format!("{document_url}/live/frame"),
        api: format!("/api/v1/artifacts/{encoded}"),
        people_api: format!("{USER_BASE_URL}/people"),
        search_api: format!("{}/people", ApiPaths::DOCUMENTS),
        sign_in: format!("{SIGN_IN}?redirect={}", urlencoding::encode(&back)),
        connectors: CONNECTORS,
        document_url,
        document_id,
    };
    Ok(render_workspace_page(
        &shell.engine,
        "drive-live",
        &page,
        Viewer::new(&shell.user, &shell.marketplace),
    ))
}
