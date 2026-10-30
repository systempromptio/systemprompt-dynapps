//! The "Documents" panel the skill and conversation pages share: the
//! documents the subject created or edited, each linking to its page in the
//! user-facing Drive.

use serde::Serialize;
use systemprompt_web_admin_core::ssr::format::{local_time, relative_time};

use crate::repositories::analysis::documents::DocumentTouchRow;

const DRIVE_URL: &str = "/drive";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DocumentTouchView {
    name: String,
    format: String,
    href: String,
    action: &'static str,
    revisions: i64,
    last_display: String,
    last_title: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub(crate) struct DocumentsPanelView {
    created: usize,
    edited: usize,
    rows: Vec<DocumentTouchView>,
}

pub(crate) fn documents_panel(rows: &[DocumentTouchRow]) -> DocumentsPanelView {
    let created = rows.iter().filter(|r| r.created).count();
    DocumentsPanelView {
        created,
        edited: rows.len() - created,
        rows: rows
            .iter()
            .map(|r| DocumentTouchView {
                name: r.name.clone(),
                format: r.format.clone(),
                href: format!(
                    "{DRIVE_URL}/{}",
                    urlencoding::encode(r.document_id.as_str())
                ),
                action: if r.created { "Created" } else { "Edited" },
                revisions: r.revisions,
                last_display: relative_time(r.last_at),
                last_title: local_time(r.last_at),
            })
            .collect(),
    }
}
