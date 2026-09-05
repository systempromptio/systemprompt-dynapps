//! Typed view model for the `/admin/users` roster.
//!
//! Strict Handlebars turns an absent field into a 500, so every name the
//! template reads is a field here and the two move together.

use serde::Serialize;
use systemprompt::identifiers::UserId;

use crate::handlers::ssr::list_view::{Pagination, ScopeFilterView};
use crate::handlers::ssr::types::{FilterChipView, SortHeaderView};

#[derive(Debug, Serialize)]
pub(crate) struct RosterContext {
    pub page: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub window_label: String,

    pub kpis: RosterKpiView,
    pub chips: Vec<FilterChipView>,
    pub role_options: Vec<RoleOptionView>,
    pub search: String,
    pub scope_filter: ScopeFilterView,

    pub sort_headers: Vec<SortHeaderView>,
    pub rows: Vec<RosterRowView>,
    pub has_rows: bool,
    pub row_count: usize,
    pub pagination: Pagination,
    pub count_label: String,

    // Why: the write controls. A project manager reads the roster but may not
    // create an account or change a role, so the buttons are absent rather
    // than present and refused.
    pub can_write: bool,
    pub role_choices: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct RosterKpiView {
    pub total: i64,
    pub active: i64,
    pub unassigned: i64,
    pub no_role: i64,
    pub inactive: i64,
    pub cost_display: String,
    pub cost_delta: String,
    pub cost_delta_dir: &'static str,
    pub requests_display: String,
    pub requests_delta: String,
    pub requests_delta_dir: &'static str,
    pub unassigned_url: String,
    pub no_role_url: String,
    pub inactive_url: String,
    pub all_url: String,
    pub filter_active: bool,
    pub unassigned_active: bool,
    pub no_role_active: bool,
    pub inactive_active: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct RoleOptionView {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct RosterRowView {
    pub user_id: UserId,
    pub name: String,
    pub initials: String,
    pub email: String,
    pub detail_url: String,
    pub roles: Vec<String>,
    pub has_roles: bool,
    pub groups: Vec<ChipView>,
    pub has_groups: bool,
    pub projects: Vec<ChipView>,
    pub has_projects: bool,
    pub last_active: Option<String>,
    pub last_active_title: String,
    pub cost_display: String,
    pub requests_display: String,
    pub tokens_display: String,
    pub is_active: bool,
    pub status_label: &'static str,
    pub status_tone: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct ChipView {
    pub id: String,
    pub label: String,
    pub href: String,
    pub tone: &'static str,
}
