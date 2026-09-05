//! Template context types for the People pages: groups, projects, and the
//! derived Unassigned bucket.
//!
//! Every displayed number is already formatted here, so the templates do no
//! arithmetic and a column cannot disagree with the total above it. Money
//! stays in microdollars and is rendered by the `formatUsd` helper, which is
//! the one place the precision rule lives.

use serde::Serialize;
use systemprompt::identifiers::UserId;

use super::super::types::SvgLineChartView;
use super::{BreadcrumbView, TabLinkView};

// Why: A label / value tile in a page's stat row.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct StatTileView {
    pub label: &'static str,
    pub value: String,
}

// Why: A small named pill: a marketplace on a group row, a group on a member
// row.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct MemberSetChipView {
    pub id: String,
    pub label: String,
}

// Why: The membership source badge. `adfs` rows are replaced wholesale at every
// sign-in, so an operator may not remove one by hand — the badge is what
// tells them why the Remove button is missing.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct SourceBadgeView {
    pub source: String,
    pub label: String,
    pub color: &'static str,
    pub title: String,
}

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
    pub range_label: &'static str,
    pub window_days: i32,
    pub groups: Vec<GroupRowView>,
    pub unattributed: UnattributedRowView,
    pub pagination: super::super::list_view::Pagination,
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

// Why: One row of the model-mix table: a share bar plus the numbers behind it.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ModelMixRowView {
    pub model: String,
    pub provider: String,
    pub requests: i64,
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub cost_microdollars: i64,
    pub share_pct: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NameCountRowView {
    pub name: String,
    pub count: i64,
    pub share_pct: i64,
}

// Why: One person on the group's spend leaderboard. It is a member-attributed
// view — a person in two groups appears on both boards in full — which is why
// the section carrying it is labelled as overlapping.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct UsageLeaderRowView {
    pub user_id: String,
    pub href: String,
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub share_pct: i64,
}

#[derive(Debug, Serialize)]
pub(crate) struct GroupOverviewView {
    pub models: Vec<ModelMixRowView>,
    pub daily_requests: SvgLineChartView,
    pub skills: Vec<NameCountRowView>,
    pub tools: Vec<NameCountRowView>,
    pub leaderboard: Vec<UsageLeaderRowView>,
}

// Why: One marketplace, and whether this group reaches it. Entitlement is an
// ordinary access-control rule on the marketplace entity keyed by the group
// dimension, so the checkbox writes a rule rather than a membership.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct MarketplaceAssignmentView {
    pub id: String,
    pub name: String,
    pub description: String,
    pub assigned: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MemberRowView {
    pub user_id: UserId,
    pub display_name: String,
    pub email: Option<String>,
    pub detail_href: String,
    pub is_active: bool,
    pub requests_30d: i64,
    pub tokens_30d: i64,
    pub cost_30d_microdollars: i64,
    pub last_active: Option<String>,
    pub sources: Vec<SourceBadgeView>,
    pub source_ad_groups: Vec<String>,
    // Why: a member the directory put here cannot be removed by hand — the
    // next sign-in would write the row straight back.
    pub can_remove: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UserOptionView {
    pub user_id: UserId,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MembersTabView {
    pub rows: Vec<MemberRowView>,
    pub addable_users: Vec<UserOptionView>,
    // Why: only the Unassigned page offers a destination select; every other
    // group page moves people with the Add member dialog instead.
    pub assign_targets: Vec<MemberSetChipView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProjectRowView {
    pub id: String,
    pub name: String,
    pub href: String,
    pub description: Option<String>,
    pub member_count: i64,
    pub group_count: i64,
    pub active_members_30d: i64,
    pub requests_30d: i64,
    pub cost_30d_microdollars: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MappingRowView {
    pub ad_group: String,
    pub source: String,
    pub can_remove: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct GroupDetailPageData {
    pub page: &'static str,
    pub title: String,
    pub group_id: String,
    pub group_name: String,
    pub description: Option<String>,
    pub is_unassigned: bool,
    pub not_found: bool,
    pub can_manage: bool,
    pub can_map: bool,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub tabs: Vec<TabLinkView>,
    pub active_tab: String,
    pub stats: Vec<StatTileView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overview: Option<GroupOverviewView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<MembersTabView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marketplaces: Option<Vec<MarketplaceAssignmentView>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access: Option<Vec<AccessSectionView>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projects: Option<Vec<ProjectRowView>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mappings: Option<Vec<MappingRowView>>,
}

// Why: One entity's effective grant for a group subject, plus the toggle state
// the editor writes back through the ACL rules API.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct AccessRowView {
    pub entity_type: String,
    pub entity_id: String,
    pub entity_name: String,
    pub effective: String,
    pub layer: String,
    pub detail: String,
    // Why: `inherit` means no rule at this group's own band — the cell is
    // whatever the wider bands decided, and clearing a rule returns to it.
    pub state: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AccessSectionView {
    pub entity_type: String,
    pub label: String,
    pub rows: Vec<AccessRowView>,
}
