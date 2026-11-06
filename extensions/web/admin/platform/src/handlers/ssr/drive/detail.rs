//! One document's block editor page, reached from the workspace name menu:
//! its page view, the block editor, the revision history and the same
//! comments, proposals and sharing panes the workspace drawer shows.
//!
//! Like the workspace, it asks the document service for the viewer's access
//! before it renders, and the editor is read-only below edit access.

use axum::Extension;
use axum::extract::Path;
use axum::response::Response;
use serde::Serialize;
use systemprompt::identifiers::{DocumentId, UserId};
use systemprompt::models::ApiPaths;
use systemprompt::models::documents::AccessLevel;

use systemprompt_web_admin_core::error::AdminHtmlResult;
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

use super::document_api;
use super::user::{DriveDocuments, USER_BASE_URL};
use super::workspace::authorize;

#[derive(Debug, Serialize)]
struct DriveDocumentPage {
    page: &'static str,
    title: String,
    breadcrumbs: Vec<BreadcrumbView>,
    document_id: DocumentId,
    documents_api: &'static str,
    document_api: String,
    preview_url: String,
    content_url: String,
    revisions_api: String,
    presence_api: String,
    projection_api: String,
    edits_api: String,
    people_api: String,
    my_access: &'static str,
    can_edit: bool,
    can_share: bool,
    user_id: UserId,
}

pub(crate) async fn drive_document_page(
    shell: Page,
    Extension(documents): Extension<DriveDocuments>,
    Path(raw_id): Path<String>,
) -> AdminHtmlResult<Response> {
    let doc = authorize(&shell, &documents, raw_id).await?;
    let api = document_api(doc.document_id.as_str());
    let page = DriveDocumentPage {
        page: "drive-document",
        breadcrumbs: vec![
            BreadcrumbView::link("Drive", USER_BASE_URL),
            BreadcrumbView::link(
                doc.name.clone(),
                format!(
                    "{USER_BASE_URL}/{}",
                    urlencoding::encode(doc.document_id.as_str())
                ),
            ),
            BreadcrumbView::current("Block editor".to_owned()),
        ],
        title: doc.name,
        documents_api: ApiPaths::DOCUMENTS,
        preview_url: format!("{api}/preview"),
        content_url: format!("{api}/content"),
        revisions_api: format!("{api}/revisions"),
        presence_api: format!("{api}/presence"),
        projection_api: format!("{api}/projection"),
        edits_api: format!("{api}/edits"),
        people_api: format!("{USER_BASE_URL}/people"),
        my_access: doc.access.as_str(),
        can_edit: doc.access >= AccessLevel::Edit,
        can_share: doc.access >= AccessLevel::Share,
        user_id: shell.user.user_id.clone(),
        document_api: api,
        document_id: doc.document_id,
    };
    Ok(render_page_with_faults(
        &shell.engine,
        "drive-document",
        &page,
        Viewer::new(&shell.user, &shell.marketplace),
        &Faults::new(),
    ))
}
