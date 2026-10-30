//! `/drive` and `/drive/{id}` — the shared document store as every signed-in
//! person sees it: their documents, the ones shared with them and the ones
//! they touched last, and one document with its editor or preview, the
//! people in it, its comments, proposals and history.
//!
//! The listing and the panels are core's document components, which read
//! `/api/v1/documents` as the viewer. The document page asks the document
//! service for the viewer's access before it renders, so a person without a
//! grant meets a 404 rather than a page shell whose panels all fail.

use std::sync::Arc;

use axum::Extension;
use axum::extract::Path;
use axum::response::Response;
use serde::Serialize;
use systemprompt::files::documents::{DocumentService, DocumentsError};
use systemprompt::identifiers::{DocumentId, TraceId};
use systemprompt::models::auth::{AuthenticatedUser, Permission};
use systemprompt::models::documents::{AccessLevel, DocumentFormat};

use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::types::UserContext;

use super::templates::{TemplateView, install_templates};

pub const USER_BASE_URL: &str = "/drive";

/// The document service the user-facing Drive authorizes page loads with.
#[derive(Clone)]
pub struct DriveDocuments(pub Arc<DocumentService>);

impl std::fmt::Debug for DriveDocuments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DriveDocuments").finish_non_exhaustive()
    }
}

#[derive(Debug, Serialize)]
struct UserDrivePage {
    page: &'static str,
    title: &'static str,
    breadcrumbs: Vec<BreadcrumbView>,
    base_url: &'static str,
    templates: Vec<TemplateView>,
}

#[derive(Debug, Serialize)]
struct UserDriveDocumentPage {
    page: &'static str,
    title: String,
    breadcrumbs: Vec<BreadcrumbView>,
    document_id: DocumentId,
    format: &'static str,
    editor: &'static str,
    readonly: bool,
    can_share: bool,
    content_url: String,
}

fn ensure_enabled(shell: &Page) -> Result<(), AdminError> {
    if shell.engine.documents_enabled() {
        Ok(())
    } else {
        Err(AdminError::NotFound(
            "Document store is disabled.".to_owned(),
        ))
    }
}

fn caller(user: &UserContext) -> AuthenticatedUser {
    let permission = if user.is_admin {
        Permission::Admin
    } else {
        Permission::User
    };
    AuthenticatedUser::new_with_roles(
        user.user_id.clone(),
        user.username.clone(),
        user.email.as_str().to_owned(),
        vec![permission],
        user.roles.clone(),
    )
}

const fn editor_for(format: DocumentFormat) -> &'static str {
    match format {
        DocumentFormat::Html | DocumentFormat::Markdown => "editor",
        DocumentFormat::Xlsx => "sheet",
        DocumentFormat::Pptx => "slides",
        _ => "preview",
    }
}

pub(crate) async fn user_drive_page(
    shell: Page,
    Extension(boot): Extension<ExtensionBoot>,
) -> AdminHtmlResult<Response> {
    ensure_enabled(&shell)?;
    let page = UserDrivePage {
        page: "my-drive",
        title: "Drive",
        breadcrumbs: vec![BreadcrumbView::current("Drive".to_owned())],
        base_url: USER_BASE_URL,
        templates: install_templates(boot.services_root())?,
    };
    Ok(render_page_with_faults(
        &shell.engine,
        "my-drive",
        &page,
        Viewer::new(&shell.user, &shell.marketplace),
        &Faults::new(),
    ))
}

pub(crate) async fn user_drive_document_page(
    shell: Page,
    Extension(documents): Extension<DriveDocuments>,
    Path(raw_id): Path<String>,
) -> AdminHtmlResult<Response> {
    ensure_enabled(&shell)?;
    let not_found = || AdminError::NotFound("Document not found.".to_owned());
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
    let format = view.document.format;
    let page = UserDriveDocumentPage {
        page: "my-drive-document",
        breadcrumbs: vec![
            BreadcrumbView::link("Drive", USER_BASE_URL),
            BreadcrumbView::current(view.document.name.clone()),
        ],
        title: view.document.name,
        format: format.as_str(),
        editor: editor_for(format),
        readonly: view.my_access < AccessLevel::Edit,
        can_share: view.my_access >= AccessLevel::Share,
        content_url: format!(
            "{}/{}/content",
            systemprompt::models::ApiPaths::DOCUMENTS,
            urlencoding::encode(document_id.as_str())
        ),
        document_id,
    };
    Ok(render_page_with_faults(
        &shell.engine,
        "my-drive-document",
        &page,
        Viewer::new(&shell.user, &shell.marketplace),
        &Faults::new(),
    ))
}
