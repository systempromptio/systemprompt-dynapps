//! Typed template context for the conversation-history page.

use serde::Serialize;
use systemprompt::identifiers::{SessionId, UserId};

use crate::handlers::ssr::list_view::Pagination;

#[derive(Debug, Serialize)]
pub(super) struct HistoryPageContext {
    pub page: &'static str,
    pub title: &'static str,
    pub search_query: String,
    pub filter_user_id: Option<String>,
    pub viewer_is_admin: bool,
    pub scope_label: String,
    pub has_rows: bool,
    pub rows: Vec<HistoryRowView>,
    pub pagination: Pagination,
}

#[derive(Debug, Serialize)]
pub(super) struct HistoryRowView {
    pub source: &'static str,
    pub is_gateway: bool,
    pub session_id: Option<SessionId>,
    pub short_id: String,
    pub title: String,
    pub user_id: UserId,
    pub is_own: bool,
    pub model: Option<String>,
    pub when_local: String,
    pub entries_counted: i64,
    pub tokens_display: String,
    pub cost_display: Option<String>,
    pub preview: Option<String>,
    pub snippet: Option<String>,
    // Why: the gateway detail page is owner-facing, so every viewer gets a
    // link; the transcript detail page is the admin session view, so only an
    // admin does.
    pub detail_url: Option<String>,
}
