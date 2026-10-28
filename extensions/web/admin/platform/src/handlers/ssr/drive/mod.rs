//! `/admin/drive` — the shared document store as an administrator sees it:
//! folders and documents, and one document's preview, revisions, grants and
//! live editors.
//!
//! The handlers render the page shell and the core API paths it reads. The
//! rows come from core's `/api/v1/documents` routes, fetched by the page
//! script with the admin session, so the document service authorizes every
//! read and this crate never queries the store's tables.

mod detail;
mod list;

pub(crate) use detail::drive_document_page;
pub(crate) use list::drive_page;

use systemprompt::models::ApiPaths;

pub(crate) const BASE_URL: &str = "/admin/drive";

fn document_api(id: &str) -> String {
    format!("{}/{}", ApiPaths::DOCUMENTS, urlencoding::encode(id))
}
