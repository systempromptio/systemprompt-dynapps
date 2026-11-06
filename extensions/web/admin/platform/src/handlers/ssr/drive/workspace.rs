//! One document in the full-screen workspace: the top bar with the name menu,
//! the navigation rail, the editor slot, the details drawer and the command
//! palette. The commands, drawer tabs and format editors are registered by
//! the page scripts, so the shell only carries the document's paths and the
//! viewer's access, which the scripts gate every command and editor on.
//!
//! The page asks the document service for the viewer's access before it
//! renders, so a person without a grant meets a 404, never a page shell.

use axum::Extension;
use axum::extract::Path;
use axum::response::Response;
use serde::Serialize;
use systemprompt::files::documents::DocumentsError;
use systemprompt::identifiers::{DocumentId, TraceId, UserId};
use systemprompt::models::ApiPaths;
use systemprompt::models::documents::AccessLevel;

use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_workspace_page};
use systemprompt_web_admin_core::ssr::page::Page;

use super::document_api;
use super::user::{DriveDocuments, USER_BASE_URL, caller};

pub(super) struct Authorized {
    pub(super) document_id: DocumentId,
    pub(super) name: String,
    pub(super) access: AccessLevel,
}

pub(super) async fn authorize(
    shell: &Page,
    documents: &DriveDocuments,
    raw_id: String,
) -> Result<Authorized, AdminError> {
    let not_found = || AdminError::NotFound("Document not found.".to_owned());
    if !shell.engine.documents_enabled() {
        return Err(AdminError::NotFound(
            "Document store is disabled.".to_owned(),
        ));
    }
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
    Ok(Authorized {
        document_id,
        name: view.document.name,
        access: view.my_access,
    })
}

#[derive(Debug, Serialize)]
struct DriveWorkspacePage {
    page: &'static str,
    title: String,
    base_url: &'static str,
    documents_api: &'static str,
    blocks_url: String,
    document_api: String,
    preview_url: String,
    content_url: String,
    revisions_api: String,
    presence_api: String,
    projection_api: String,
    access_api: String,
    people_api: String,
    my_access: &'static str,
    can_edit: bool,
    can_share: bool,
    user_id: UserId,
}

pub(crate) async fn drive_workspace_page(
    shell: Page,
    Extension(documents): Extension<DriveDocuments>,
    Path(raw_id): Path<String>,
) -> AdminHtmlResult<Response> {
    let doc = authorize(&shell, &documents, raw_id).await?;
    let api = document_api(doc.document_id.as_str());
    let page = DriveWorkspacePage {
        page: "drive-workspace",
        title: doc.name,
        base_url: USER_BASE_URL,
        documents_api: ApiPaths::DOCUMENTS,
        blocks_url: format!(
            "{USER_BASE_URL}/{}/blocks",
            urlencoding::encode(doc.document_id.as_str())
        ),
        preview_url: format!("{api}/preview"),
        content_url: format!("{api}/content"),
        revisions_api: format!("{api}/revisions"),
        presence_api: format!("{api}/presence"),
        projection_api: format!("{api}/projection"),
        access_api: format!("{api}/access"),
        people_api: format!("{USER_BASE_URL}/people"),
        my_access: doc.access.as_str(),
        can_edit: doc.access >= AccessLevel::Edit,
        can_share: doc.access >= AccessLevel::Share,
        user_id: shell.user.user_id.clone(),
        document_api: api,
    };
    Ok(render_workspace_page(
        &shell.engine,
        "drive-workspace",
        &page,
        Viewer::new(&shell.user, &shell.marketplace),
    ))
}
