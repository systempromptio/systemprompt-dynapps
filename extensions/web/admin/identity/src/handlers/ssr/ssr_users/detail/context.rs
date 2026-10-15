//! Typed view model for `/admin/users/{id}`.
//!
//! One context type carries every tab, but only the active tab's fields are
//! populated: the handler loads what the reader asked for and nothing else, so
//! opening the Identity tab does not pay for a usage aggregate.

use serde::Serialize;
use systemprompt::identifiers::{ContextId, SessionId, UserId};

use crate::handlers::ssr::types::{MembershipChoiceView, RoleChoiceView};
use systemprompt_web_admin_core::ssr::history_rows::HistoryRowView;
use systemprompt_web_admin_core::ssr::list_view::Pagination;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

#[derive(Debug, Serialize)]
pub(crate) struct UserDetailContext {
    pub page: &'static str,
    pub title: String,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub tabs: Vec<DetailTabView>,
    pub tab: &'static str,

    pub header: UserHeaderView,
    pub kpis: DetailKpiView,

    pub can_write: bool,
    pub is_self: bool,

    pub identity: Option<IdentityTabView>,
    pub conversations: Option<UserConversationsTabView>,
    pub membership: Option<MembershipTabView>,
    pub access: Option<AccessTabView>,
    pub devices: Option<DevicesTabView>,
    pub sessions: Option<UserSessionsTabView>,
    pub usage: Option<UsageTabView>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<systemprompt_web_admin_core::export::view::ExportView>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DetailTabView {
    pub slug: &'static str,
    pub label: &'static str,
    pub href: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct UserHeaderView {
    pub user_id: UserId,
    pub name: String,
    pub initials: String,
    pub email: String,
    pub roles: Vec<String>,
    pub has_roles: bool,
    pub is_active: bool,
    pub status_label: &'static str,
    pub status_tone: &'static str,
    pub created_at: String,
    pub last_active: String,
    pub primary_group: String,
    pub primary_project: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct DetailKpiView {
    pub requests_display: String,
    pub cost_display: String,
    pub tokens_display: String,
    pub sessions: i64,
    pub devices: i64,
    pub groups: usize,
    pub projects: usize,
    pub membership_url: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct IdentityTabView {
    pub display_name: String,
    pub email: String,
    pub is_active: bool,
    pub created_at: String,
    pub role_choices: Vec<RoleChoiceView>,
    pub roles_valid_until_day: String,
    pub adfs_groups: Vec<String>,
    pub has_adfs_groups: bool,
    pub idp_issuer: String,
    pub external_sub: String,
    pub linked_at: String,
    pub slack_user_id: String,
    pub share_token_version: i32,
}

#[derive(Debug, Serialize)]
pub(crate) struct MembershipTabView {
    pub group_choices: Vec<MembershipChoiceView>,
    pub project_choices: Vec<MembershipChoiceView>,
    pub primary_group_options: Vec<ScopeDefaultOptionView>,
    pub primary_project_options: Vec<ScopeDefaultOptionView>,
    pub scope_source: String,
    pub scope_source_is_manual: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct ScopeDefaultOptionView {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct AccessTabView {
    pub available: bool,
    pub rules_available: bool,
    pub overview: super::access_overview::AccessOverview,
    pub has_groups: bool,
    pub sections: Vec<UserAccessSectionView>,
}

#[derive(Debug, Serialize)]
pub(crate) struct UserAccessSectionView {
    pub entity_type: String,
    pub label: String,
    pub rows: Vec<UserAccessRowView>,
    pub has_rows: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct UserAccessRowView {
    pub entity_type: String,
    pub entity_id: String,
    pub entity_name: String,
    pub effective: String,
    pub effective_tone: &'static str,
    pub layer: String,
    pub detail: String,
    pub state: &'static str,
    pub rule_id: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct DevicesTabView {
    pub rows: Vec<DeviceRowView>,
    pub has_rows: bool,
    pub count: usize,
    pub active_count: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct DeviceRowView {
    pub id: String,
    pub kind: String,
    pub kind_label: &'static str,
    pub label: String,
    pub detail: String,
    pub created_at: String,
    pub last_seen: String,
    pub status_label: &'static str,
    pub status_tone: &'static str,
    pub is_active: bool,
    pub revocable: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct UserSessionsTabView {
    pub rows: Vec<UserSessionRowView>,
    pub has_rows: bool,
    pub pagination: Pagination,
    pub live_count: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct UserSessionRowView {
    pub session_id: SessionId,
    pub short_id: String,
    pub detail_url: String,
    pub source: String,
    pub ip: String,
    pub user_agent: String,
    pub requests: i32,
    pub started_at: String,
    pub last_activity: String,
    pub status_label: &'static str,
    pub status_tone: &'static str,
    pub revocable: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct UserConversationsTabView {
    pub rows: Vec<HistoryRowView>,
    pub has_rows: bool,
    pub pagination: Pagination,
}

#[derive(Debug, Serialize)]
pub(crate) struct UsageTabView {
    pub window_label: String,
    pub requests: i64,
    pub conversations: i64,
    pub turns: i64,
    pub tool_calls: i64,
    pub side_calls: i64,
    pub side_call_cost_display: String,
    pub failed: i64,
    pub tokens_display: String,
    pub cost_display: String,
    pub first_request_at: String,
    pub last_request_at: String,
    pub latest: Option<LatestConversationView>,
    pub models: Vec<UserModelRowView>,
    pub has_models: bool,
    pub conversations_url: String,
    pub log_url: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct UserModelRowView {
    pub model: String,
    pub requests: i64,
    pub tokens_display: String,
    pub cost_display: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct LatestConversationView {
    pub conversation_title: String,
    pub url: String,
    pub context_id: ContextId,
    pub last_relative: String,
    pub last_at: String,
    pub model: String,
    pub turns: i64,
    pub tool_calls: i64,
    pub cost_display: String,
}
