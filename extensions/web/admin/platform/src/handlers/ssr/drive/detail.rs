//! One document: its preview, a download of the head revision, the revision
//! history, the grants on it and who is editing it now.

use axum::extract::Path;
use axum::response::Response;
use serde::Serialize;
use systemprompt::identifiers::DocumentId;

use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

use super::{BASE_URL, document_api};

#[derive(Debug, Serialize)]
struct DriveDocumentPage {
    page: &'static str,
    title: &'static str,
    breadcrumbs: Vec<BreadcrumbView>,
    document_id: DocumentId,
    document_api: String,
    preview_url: String,
    content_url: String,
    revisions_api: String,
    grants_api: String,
    presence_api: String,
}

pub(crate) async fn drive_document_page(
    shell: Page,
    Path(raw_id): Path<String>,
) -> AdminHtmlResult<Response> {
    if !shell.engine.documents_enabled() {
        return Err(AdminError::NotFound("Document store is disabled.".to_owned()).into());
    }
    if !shell.user.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let document_id =
        DocumentId::try_new(raw_id).map_err(|e| AdminError::invalid("invalid document id", e))?;
    let api = document_api(document_id.as_str());
    let page = DriveDocumentPage {
        page: "drive-document",
        title: "Document",
        breadcrumbs: vec![
            BreadcrumbView::link("Drive", BASE_URL),
            BreadcrumbView::current("Document".to_owned()),
        ],
        preview_url: format!("{api}/preview"),
        content_url: format!("{api}/content"),
        revisions_api: format!("{api}/revisions"),
        grants_api: format!("{api}/grants"),
        presence_api: format!("{api}/presence"),
        document_api: api,
        document_id,
    };
    Ok(render_page_with_faults(
        &shell.engine,
        "drive-document",
        &page,
        Viewer::new(&shell.user, &shell.marketplace),
        &Faults::new(),
    ))
}
