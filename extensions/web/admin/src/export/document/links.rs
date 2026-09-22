//! What a page hands its template so the document export buttons render.

use serde::Serialize;
use systemprompt::identifiers::{ContextId, SessionId};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DocumentExportView {
    pub json_href: String,
    pub markdown_href: String,
    // Why: a set — the page's filtered list, a ticked selection, a session —
    // leaves as JSON Lines; a single conversation has no such link.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsonl_href: Option<String>,
    pub label: &'static str,
}

impl DocumentExportView {
    // Why: one conversation from any of its reader pages.
    pub(crate) fn conversation(context_id: &ContextId) -> Self {
        let id = urlencoding::encode(context_id.as_str());
        Self {
            json_href: format!("/admin/export/conversation/{id}?format=json"),
            markdown_href: format!("/admin/export/conversation/{id}?format=markdown"),
            jsonl_href: None,
            label: "Export full conversation",
        }
    }

    // Why: every conversation a session holds.
    pub(crate) fn session(session: &SessionId) -> Self {
        let id = urlencoding::encode(session.as_str());
        Self {
            json_href: String::new(),
            markdown_href: String::new(),
            jsonl_href: Some(format!("/admin/export/conversations?session_id={id}")),
            label: "Export full conversations",
        }
    }

    // Why: the list a page is showing, carrying the page's own filters so the
    // file answers the question the page was asked.
    pub(crate) fn filtered(source: &str, query: &str) -> Self {
        let query = query.trim_start_matches('?').trim_matches('&');
        let sep = if query.is_empty() { "" } else { "&" };
        Self {
            json_href: String::new(),
            markdown_href: String::new(),
            jsonl_href: Some(format!(
                "/admin/export/conversations?source={source}{sep}{query}"
            )),
            label: "Export full conversations",
        }
    }
}
