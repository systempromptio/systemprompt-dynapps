//! The groups listing's own context types: one row, the remainder that
//! completes it, and the controls above it.
//!
//! They live apart from the detail page's types because the listing is where
//! the exclusive-attribution contract is visible — the rows plus
//! [`UnattributedRowView`] reproduce the instance total — and that claim is
//! easier to keep true when the shapes carrying it sit together.

use serde::Serialize;

use super::super::list_view::Pagination;
use super::{BreadcrumbView, MemberSetChipView};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GroupRowView {
    pub id: String,
    pub name: String,
    pub href: String,
    pub description: Option<String>,
    pub member_count: i64,
    pub active_members_30d: i64,
    pub marketplaces: Vec<MemberSetChipView>,
    pub project_count: i64,
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    // Why: the busiest model a group's own traffic used, which is the one
    // question a cost column always provokes and would otherwise cost a click.
    pub top_model: Option<String>,
    pub top_model_requests: i64,
    // Why: this group's share of the instance's requests in the window. Under
    // exclusive attribution the shares are a partition, so they are readable
    // as proportions rather than as overlapping fractions.
    pub share_pct: i64,
    pub source: String,
    pub source_label: &'static str,
    pub is_unassigned: bool,
}

// Why: The remainder row. Under exclusive attribution every group's traffic
// plus this equals the instance, so it is a row of the same table rather than
// a footnote: dropping it would leave a listing that silently fails to add up.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct UnattributedRowView {
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub share_pct: i64,
    pub people: i64,
}

// Why: One header tile. Named for the page rather than shared, because two
// other pages already define a `KpiView` of their own and the duplicate-type
// gate reads names, not shapes.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct GroupKpiView {
    pub label: &'static str,
    pub value: String,
    pub sub: String,
    pub tone: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GroupSortHeaderView {
    pub label: &'static str,
    pub class: &'static str,
    pub hint: &'static str,
    pub url: String,
    pub active: bool,
    pub aria_sort: &'static str,
    pub indicator: &'static str,
}

// Why: One entry of the window picker. `data-scope-range` is the hook the
// e2e harness drives every range control through.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct RangeLinkView {
    pub label: &'static str,
    pub value: &'static str,
    pub url: String,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct GroupsPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub kpis: Vec<GroupKpiView>,
    pub sort_headers: Vec<GroupSortHeaderView>,
    pub ranges: Vec<RangeLinkView>,
    pub sources: Vec<RangeLinkView>,
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
    // Why: destinations for the header's "Map a directory group" dialog, which
    // is where a mapping is created from — the group it lands in is a field of
    // the form rather than a page you must first navigate to.
    pub group_options: Vec<MemberSetChipView>,
    // Why: people with no primary group at all. Exclusive attribution has
    // nowhere to file their spend, so the page says how many there are and
    // offers the recompute rather than quietly reporting a smaller instance.
    pub unkeyed_people: i64,
}
