//! Typed template context for the conversation-history page.

use serde::Serialize;

use systemprompt_web_admin_core::ssr::history_rows::HistoryRowView;
use systemprompt_web_admin_core::ssr::list_view::Pagination;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::ssr::types::tabs::TabLinkView;

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
    pub export: systemprompt_web_admin_core::export::view::ExportView,
}

#[derive(Debug, Serialize)]
pub(super) struct HiddenInputView {
    pub name: &'static str,
    pub value: String,
}
