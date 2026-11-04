//! `/drive`: the one listing every signed-in person sees — one folder's
//! subfolders and documents, or the recent, shared and own lists — with the
//! controls only an administrator may use (a folder's agent edit policy)
//! rendered for administrators alone.
//!
//! The rows are read by the page script from `/api/v1/documents` as the
//! viewer. The handler asks the document service for the open folder and
//! each of its ancestors as the viewer, so a folder the viewer cannot see
//! is a 404 and an ancestor they cannot see is left out of the trail.

use axum::Extension;
use axum::extract::Query;
use axum::response::Response;
use serde::{Deserialize, Serialize};
use systemprompt::files::documents::DocumentsError;
use systemprompt::identifiers::{DocumentFolderId, TraceId};
use systemprompt::models::ApiPaths;
use systemprompt::models::auth::AuthenticatedUser;

use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

use super::BASE_URL;
use super::templates::{TemplateView, install_templates};
use super::user::{DriveDocuments, caller, ensure_enabled};

#[derive(Debug, Deserialize)]
pub(crate) struct DriveQuery {
    folder: Option<String>,
}

#[derive(Debug, Serialize)]
struct DrivePage {
    page: &'static str,
    title: String,
    breadcrumbs: Vec<BreadcrumbView>,
    folder_id: Option<DocumentFolderId>,
    documents_api: &'static str,
    folders_api: &'static str,
    base_url: &'static str,
    is_admin: bool,
    templates: Vec<TemplateView>,
}

fn folder_link(id: &DocumentFolderId) -> String {
    format!("{BASE_URL}?folder={}", urlencoding::encode(id.as_str()))
}

const fn hidden(error: &DocumentsError) -> bool {
    matches!(
        error,
        DocumentsError::Forbidden { .. }
            | DocumentsError::NotFound { .. }
            | DocumentsError::Trashed { .. }
    )
}

async fn folder_trail(
    documents: &DriveDocuments,
    who: &AuthenticatedUser,
    folder: &DocumentFolderId,
) -> Result<(String, Vec<BreadcrumbView>), AdminError> {
    let trace = TraceId::generate();
    let not_found = || AdminError::NotFound("Folder not found.".to_owned());
    let open = documents
        .0
        .folder_access_of(who, folder, false, &trace)
        .await
        .map_err(|error| {
            if hidden(&error) {
                not_found()
            } else {
                AdminError::internal(error)
            }
        })?
        .target;
    let mut trail = vec![BreadcrumbView::link("Drive", BASE_URL)];
    for ancestor in &open.ancestor_ids {
        match documents
            .0
            .folder_access_of(who, ancestor, false, &trace)
            .await
        {
            Ok(access) => trail.push(BreadcrumbView::link(
                access.target.name,
                folder_link(ancestor),
            )),
            Err(error) if hidden(&error) => {},
            Err(error) => return Err(AdminError::internal(error)),
        }
    }
    trail.push(BreadcrumbView::current(open.name.clone()));
    Ok((open.name, trail))
}

pub(crate) async fn drive_page(
    shell: Page,
    Extension(boot): Extension<ExtensionBoot>,
    Extension(documents): Extension<DriveDocuments>,
    Query(query): Query<DriveQuery>,
) -> AdminHtmlResult<Response> {
    ensure_enabled(&shell)?;
    let folder_id = query
        .folder
        .filter(|raw| !raw.is_empty())
        .map(DocumentFolderId::try_new)
        .transpose()
        .map_err(|_invalid| AdminError::NotFound("Folder not found.".to_owned()))?;
    let (title, breadcrumbs) = match &folder_id {
        Some(folder) => folder_trail(&documents, &caller(&shell.user), folder).await?,
        None => (
            "Drive".to_owned(),
            vec![BreadcrumbView::current("Drive".to_owned())],
        ),
    };
    let page = DrivePage {
        page: "drive",
        title,
        breadcrumbs,
        folder_id,
        documents_api: ApiPaths::DOCUMENTS,
        folders_api: ApiPaths::DOCUMENT_FOLDERS,
        base_url: BASE_URL,
        is_admin: shell.user.is_admin,
        templates: install_templates(boot.services_root())?,
    };
    Ok(render_page_with_faults(
        &shell.engine,
        "drive",
        &page,
        Viewer::new(&shell.user, &shell.marketplace),
        &Faults::new(),
    ))
}
