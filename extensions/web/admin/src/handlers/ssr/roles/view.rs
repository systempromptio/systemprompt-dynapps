//! Template context for `/admin/roles`.
//!
//! The page renders two tables over one read: every (role, holder) pair, and
//! every entity a role opens. Both are serialised flat — the role card is a
//! KPI tile that filters the assignment table rather than a container the
//! rows live inside, so a person holding three roles is three rows an
//! operator can sort by cost, not three entries buried in three cards.

use serde::Serialize;

use crate::handlers::ssr::list_view::Pagination;
use crate::handlers::ssr::types::SortHeaderView;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RoleCardView {
    pub id: String,
    pub label: String,
    pub member_count: usize,
    pub manual_count: usize,
    pub directory_count: usize,
    pub entitlement_count: usize,
    pub tone: &'static str,
    pub href: String,
    pub active: bool,
    pub sub: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RoleMemberView {
    pub user_id: String,
    pub display_name: String,
    pub email: String,
    pub role: String,
    pub role_label: String,
    pub source: &'static str,
    pub source_tone: &'static str,
    pub is_manual: bool,
    pub is_active: bool,
    pub status: &'static str,
    pub status_tone: &'static str,
    pub requests_30d: i64,
    pub cost_30d_microdollars: i64,
    pub href: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RoleEntitlementView {
    pub role: String,
    pub role_label: String,
    pub entity_type: String,
    pub entity_type_label: String,
    pub entity_id: String,
    pub access: String,
    pub access_tone: &'static str,
    pub default_included: bool,
    pub default_label: &'static str,
    pub href: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SelectOption {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct KpiView {
    pub label: &'static str,
    pub value: String,
    pub sub: String,
    pub tone: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct RolesPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub can_write: bool,
    pub can_grant_platform_admin: bool,
    pub kpis: Vec<KpiView>,
    pub cards: Vec<RoleCardView>,
    pub members: Vec<RoleMemberView>,
    pub member_total: i64,
    pub entitlements: Vec<RoleEntitlementView>,
    pub entitlement_total: usize,
    pub pagination: Pagination,
    pub sort_headers: Vec<SortHeaderView>,
    pub role_options: Vec<SelectOption>,
    pub source_options: Vec<SelectOption>,
    pub status_options: Vec<SelectOption>,
    pub search: String,
    pub known_roles: Vec<SelectOption>,
    pub filters_applied: bool,
    pub clear_url: &'static str,
}
