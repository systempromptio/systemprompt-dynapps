//! Typed template-context structs for the conversations list page
//! (`skills-contexts.hbs`).
//!
//! `ConversationItemView` is shared by the flat "All" tab and the nested rows
//! inside each `UserSummaryView` on the "By user" tab.

use serde::Serialize;
use systemprompt::identifiers::{ContextId, UserId};

use systemprompt_web_admin_core::ssr::format::ClientChipView;
use systemprompt_web_admin_core::ssr::list_view::{Pagination, ScopeFilterView};
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::ssr::types::table::SortHeaderView;
use systemprompt_web_admin_core::ssr::types::tabs::TabLinkView;

#[derive(Debug, Serialize)]
pub(super) struct ContextsPageContext {
    pub(super) page: &'static str,
    pub(super) title: &'static str,
    pub(super) breadcrumbs: Vec<BreadcrumbView>,
    pub(super) conversations: Vec<ConversationItemView>,
    pub(super) user_summaries: Vec<UserSummaryView>,
    pub(super) users_for_filter: Vec<UserForFilterView>,
    pub(super) models: Vec<ModelOptionView>,
    pub(super) kpis: PageKpisView,
    pub(super) filter: FilterView,
    pub(super) scope_filter: ScopeFilterView,
    pub(super) view_tabs: Vec<TabLinkView>,
    pub(super) view_is_users: bool,
    pub(super) view_is_all: bool,
    pub(super) pagination: Pagination,
    pub(super) sort_headers: ContextsSortHeaders,
    pub(super) total_count: i64,
    pub(super) count_label: String,
    pub(super) has_conversations: bool,
    pub(super) has_user_summaries: bool,
    pub(super) show_side: bool,
    pub(super) side_toggle_url: String,
    pub(super) export: systemprompt_web_admin_core::export::view::ExportView,
}

#[derive(Debug, Serialize)]
pub(super) struct ContextsSortHeaders {
    pub(super) activity: SortHeaderView,
    pub(super) turns: SortHeaderView,
    pub(super) tokens: SortHeaderView,
    pub(super) cost: SortHeaderView,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ConversationItemView {
    pub(super) context_id: ContextId,
    pub(super) detail_url: String,
    // Why: Handlebars — the layout's `title=` hash shadows a `title` field at
    // every template depth.
    pub(super) conversation_title: String,
    pub(super) user_id: Option<UserId>,
    pub(super) user_label: String,
    pub(super) user_url: Option<String>,
    pub(super) model: Option<String>,
    pub(super) client: ClientChipView,
    pub(super) turn_count: i64,
    pub(super) side_call_count: i64,
    pub(super) tool_call_count: i64,
    pub(super) error_count: i64,
    pub(super) tokens_display: String,
    pub(super) tokens_title: String,
    pub(super) cost_display: String,
    pub(super) last_at: Option<String>,
    pub(super) last_relative: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct UserSummaryView {
    pub(super) user_id: UserId,
    pub(super) user_label: String,
    pub(super) user_url: String,
    pub(super) conversation_count: i64,
    pub(super) tokens_display: String,
    pub(super) all_conversations_url: String,
    pub(super) turn_count: i64,
    pub(super) side_call_count: i64,
    pub(super) cost_display: String,
    pub(super) last_at: Option<String>,
    pub(super) last_relative: Option<String>,
    pub(super) latest: Option<ConversationItemView>,
    pub(super) models_primary: Vec<ModelChipView>,
    pub(super) models_more: usize,
    pub(super) models_more_list: String,
    pub(super) clients: Vec<ClientChipView>,
    pub(super) conversations: Vec<ConversationItemView>,
    pub(super) has_conversations: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct ModelChipView {
    pub(super) model: String,
    pub(super) requests: i64,
}

#[derive(Debug, Serialize)]
pub(super) struct UserForFilterView {
    pub(super) user_id: UserId,
    pub(super) display_name: Option<String>,
    pub(super) selected: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct ModelOptionView {
    pub(super) model: String,
    pub(super) selected: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct PageKpisView {
    pub(super) conversations: i64,
    pub(super) users: i64,
    pub(super) turns: i64,
    pub(super) tool_calls: i64,
    pub(super) side_calls: i64,
    pub(super) side_call_cost_display: String,
    pub(super) tokens_display: String,
    pub(super) cost_display: String,
}

// Why: Handlebars strict mode errors on an absent key, so every field
// serializes (empty string when unset) for `{{filter.q}}` and siblings.
#[derive(Debug, Serialize)]
pub(super) struct FilterView {
    pub(super) q: String,
    pub(super) since: String,
    pub(super) view: String,
    pub(super) group: String,
    pub(super) project: String,
    pub(super) side: String,
}
