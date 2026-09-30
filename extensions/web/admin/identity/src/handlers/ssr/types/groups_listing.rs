//! The groups listing's own context types: one row, the remainder that
//! completes it, and the controls above it.
//!
//! They live apart from the detail page's types because the listing is where
//! the exclusive-attribution contract is visible — the rows plus
//! [`UnattributedRowView`] reproduce the instance total — and that claim is
//! easier to keep true when the shapes carrying it sit together.

use serde::Serialize;

use super::MemberSetChipView;
use systemprompt_web_admin_core::ssr::list_view::Pagination;
use systemprompt_web_admin_core::ssr::sync_plane::PlaneCardView;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::ssr::types::table::SortHeaderView;
use systemprompt_web_admin_core::ssr::types::tabs::TabLinkView;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GroupRowView {
    pub id: String,
    pub name: String,
    pub href: String,
    pub description: Option<String>,
    pub member_count: i64,
    pub active_members_30d: i64,
    pub marketplaces_title: String,
    pub marketplace_count: i64,
    pub project_count: i64,
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub top_model: Option<String>,
    pub top_model_short: Option<String>,
    pub top_model_requests: i64,
    pub source: String,
    pub source_label: &'static str,
    pub source_display: &'static str,
    pub source_abbrev: &'static str,
    pub is_unassigned: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UnattributedRowView {
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub share_pct: i64,
    pub people: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GroupKpiView {
    pub label: &'static str,
    pub value: String,
    // Why: `note`, not `sub`. `sub` is a registered Handlebars helper, so a
    // bare `{{sub}}` mustache calls it with no arguments and prints 0 instead
    // of the field. No view struct here is allowed to be named `sub`.
    pub note: String,
    pub tone: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FilterLinkView {
    pub label: &'static str,
    pub value: &'static str,
    pub url: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GroupSortHeaders {
    pub name: SortHeaderView,
    pub source: SortHeaderView,
    pub members: SortHeaderView,
    pub active: SortHeaderView,
    pub projects: SortHeaderView,
    pub model: SortHeaderView,
    pub requests: SortHeaderView,
    pub tokens: SortHeaderView,
    pub cost: SortHeaderView,
}

#[derive(Debug, Serialize)]
pub(crate) struct GroupsPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub tabs: Vec<TabLinkView>,
    pub sync: Option<PlaneCardView>,
    pub kpis: Vec<GroupKpiView>,
    pub sort_headers: GroupSortHeaders,
    pub ranges: Vec<FilterLinkView>,
    pub sources: Vec<FilterLinkView>,
    pub source_filter: String,
    pub range_label: &'static str,
    pub toolbar_count: String,
    pub window_days: i32,
    pub groups: Vec<GroupRowView>,
    pub unattributed: UnattributedRowView,
    pub pagination: Pagination,
    pub total_groups: i64,
    pub can_manage: bool,
    pub can_map: bool,
    pub group_options: Vec<MemberSetChipView>,
    pub unkeyed_people: i64,
    pub export: systemprompt_web_admin_core::export::view::ExportView,
}
