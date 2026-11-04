//! The Drive pages, at `/drive` for every signed-in person: one listing of
//! folders and documents, one document in the full-screen workspace with its
//! revisions, grants, comments, proposals and live editors, and the block
//! editor reached from its name menu. Administrators see the same pages with
//! the controls only they may use.
//!
//! The handlers render the page shell and the core API paths it reads. The
//! rows come from core's `/api/v1/documents` routes, fetched by the page
//! script as the viewer, so the document service authorizes every read and
//! this crate never queries the store's tables.

mod detail;
mod list;
mod people;
mod templates;
mod user;
mod workspace;

pub(crate) use detail::drive_document_page;
pub(crate) use list::drive_page;
pub(crate) use people::drive_people;
pub(crate) use user::caller as drive_caller;
pub use user::{DriveDocuments, USER_BASE_URL};
pub(crate) use workspace::drive_workspace_page;

use systemprompt::models::ApiPaths;

pub(crate) const BASE_URL: &str = USER_BASE_URL;

fn document_api(id: &str) -> String {
    format!("{}/{}", ApiPaths::DOCUMENTS, urlencoding::encode(id))
}
