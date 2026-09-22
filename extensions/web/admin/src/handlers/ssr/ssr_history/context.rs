//! Typed template context for the conversation-history page.

use serde::Serialize;
use systemprompt::identifiers::UserId;

use crate::handlers::ssr::format::ClientChipView;
use crate::handlers::ssr::list_view::Pagination;
use crate::handlers::ssr::types::{BreadcrumbView, TabLinkView};

#[derive(Debug, Serialize)]
pub(super) struct HistoryPageContext {
    pub page: &'static str,
    pub title: &'static str,
    pub search_query: String,
    pub filter_user_id: Option<String>,
    pub scope_label: String,
    pub has_rows: bool,
    pub rows: Vec<HistoryRowView>,
    pub show_side: bool,
    pub side_toggle_url: String,
    pub window_links: Vec<TabLinkView>,
    pub window_label: String,
    // Why: the search forms submit with GET, so the window rides along as
    // hidden inputs or a search would silently widen to all time.
    pub window_inputs: Vec<HiddenInputView>,
    pub base_url: &'static str,
    pub pagination: Pagination,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub export: crate::export::ExportView,
}

#[derive(Debug, Serialize)]
pub(super) struct HiddenInputView {
    pub name: &'static str,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct HistoryRowView {
    // Why: the chip names the client that recorded the conversation
    // (Claude Code, OpenCode, an API client…) and its tone says how strongly
    // the gateway evidenced it.
    pub source: ClientChipView,
    pub source_title: String,
    pub short_id: String,
    // Why: the row's selection key — the id the bulk bar's export and
    // full-record download read.
    pub context_id: String,
    // Why: named for what it is. A partial hash parameter shadows a field of
    // the same name at every depth (the layout now takes `page_title`, but
    // the row name stays specific so no later hash can collide with it).
    pub conversation_title: String,
    pub user_id: UserId,
    pub user_label: String,
    pub is_own: bool,
    pub model: Option<String>,
    pub when_relative: String,
    pub when_at: String,
    pub entries_counted: i64,
    pub side_call_count: i64,
    pub tokens_display: String,
    pub tokens_title: String,
    pub cost_display: String,
    pub detail_url: String,
    // Why: the session view is admin-gated, so only an admin gets the link
    // to the client session the conversation ran in.
    pub session_url: Option<String>,
}
