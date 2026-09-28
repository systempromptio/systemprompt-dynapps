//! Typed template context for the conversation-history page.

use serde::Serialize;
use systemprompt::identifiers::{ContextId, UserId};

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
    pub source: ClientChipView,
    pub source_title: String,
    pub short_id: String,
    pub context_id: ContextId,
    // Why: a Handlebars partial hash parameter shadows a same-named field at
    // every depth, so this row field keeps a name no hash will reuse.
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
    pub session_url: Option<String>,
}
