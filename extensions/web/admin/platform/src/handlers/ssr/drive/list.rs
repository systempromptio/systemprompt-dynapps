//! The drive listing: one folder's subfolders and documents.

use axum::extract::Query;
use axum::response::Response;
use serde::{Deserialize, Serialize};
use systemprompt::identifiers::DocumentFolderId;
use systemprompt::models::ApiPaths;

use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

use super::BASE_URL;

#[derive(Debug, Deserialize)]
pub(crate) struct DriveQuery {
    folder: Option<String>,
}

#[derive(Debug, Serialize)]
struct DrivePage {
    page: &'static str,
    title: &'static str,
    breadcrumbs: Vec<BreadcrumbView>,
    folder_id: Option<DocumentFolderId>,
    list_api: String,
    documents_api: &'static str,
    base_url: &'static str,
}

pub(crate) async fn drive_page(
    shell: Page,
    Query(query): Query<DriveQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.engine.documents_enabled() {
        return Err(AdminError::NotFound("Document store is disabled.".to_owned()).into());
    }
    if !shell.user.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let folder_id = query
        .folder
        .filter(|raw| !raw.is_empty())
        .map(DocumentFolderId::try_new)
        .transpose()
        .map_err(|e| AdminError::invalid("invalid folder id", e))?;
    let list_api = folder_id.as_ref().map_or_else(
        || ApiPaths::DOCUMENTS.to_owned(),
        |id| {
            format!(
                "{}?folder={}",
                ApiPaths::DOCUMENTS,
                urlencoding::encode(id.as_str())
            )
        },
    );
    let mut breadcrumbs = vec![BreadcrumbView::link("Drive", BASE_URL)];
    if folder_id.is_some() {
        breadcrumbs.push(BreadcrumbView::current("Folder".to_owned()));
    }
    let page = DrivePage {
        page: "drive",
        title: "Drive",
        breadcrumbs,
        folder_id,
        list_api,
        documents_api: ApiPaths::DOCUMENTS,
        base_url: BASE_URL,
    };
    Ok(render_page_with_faults(
        &shell.engine,
        "drive",
        &page,
        Viewer::new(&shell.user, &shell.marketplace),
        &Faults::new(),
    ))
}
