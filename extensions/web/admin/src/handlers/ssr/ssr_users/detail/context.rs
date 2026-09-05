//! Typed view model for `/admin/users/{id}`.
//!
//! One context type carries every tab, but only the active tab's fields are
//! populated: the handler loads what the reader asked for and nothing else, so
//! opening the Identity tab does not pay for a usage aggregate.

use serde::Serialize;
use systemprompt::identifiers::{SessionId, UserId};

use crate::handlers::ssr::list_view::Pagination;
use crate::handlers::ssr::types::{BreadcrumbView, MembershipChoiceView, RoleChoiceView};

#[derive(Debug, Serialize)]
pub(crate) struct UserDetailContext {
    pub page: &'static str,
    pub title: String,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub tabs: Vec<DetailTabView>,
    pub tab: &'static str,

    pub header: UserHeaderView,
    pub kpis: DetailKpiView,

    // Why: the write controls. A project manager reads this page; only an admin
    // may change what it shows.
    pub can_write: bool,
    // Why: an admin must not drop their own last role from this form — the API
    // refuses it, and offering the control anyway turns a guardrail into an
    // error toast.
    pub is_self: bool,

    pub identity: Option<IdentityTabView>,
    pub membership: Option<MembershipTabView>,
    pub devices: Option<DevicesTabView>,
    pub sessions: Option<UserSessionsTabView>,
    pub usage: Option<UsageTabView>,
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
}

#[derive(Debug, Serialize)]
pub(crate) struct IdentityTabView {
    pub display_name: String,
    pub email: String,
    pub is_active: bool,
    pub created_at: String,
    pub role_choices: Vec<RoleChoiceView>,
    pub adfs_groups: Vec<String>,
    pub has_adfs_groups: bool,
    pub idp_issuer: String,
    pub external_sub: String,
    pub linked_at: String,
    pub slack_user_id: String,
    pub salesforce_username: String,
    pub share_token_version: i32,
}

#[derive(Debug, Serialize)]
pub(crate) struct MembershipTabView {
    pub group_choices: Vec<MembershipChoiceView>,
    pub project_choices: Vec<MembershipChoiceView>,
    // Why: the exclusive-attribution key. Every cost total on the group and
    // project pages counts this person exactly once, under these two ids, so
    // this editor is the only place that number can be moved.
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
pub(crate) struct UsageTabView {
    pub window_label: String,
    pub requests: i64,
    pub conversations: i64,
    pub failed: i64,
    pub tokens_display: String,
    pub cost_display: String,
    pub first_request_at: String,
    pub last_request_at: String,
    pub models: Vec<UserModelRowView>,
    pub has_models: bool,
    pub conversations_rows: Vec<ConversationRowView>,
    pub has_conversations: bool,
    pub conversations_url: String,
    pub log_url: String,
    pub commits: Vec<CommitRowView>,
    pub has_commits: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct UserModelRowView {
    pub model: String,
    pub requests: i64,
    pub tokens_display: String,
    pub cost_display: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConversationRowView {
    pub title: String,
    pub short_id: String,
    pub url: String,
    pub model: String,
    pub requests: i64,
    pub messages: i64,
    pub errors: i64,
    pub has_errors: bool,
    pub cost_display: String,
    pub last_seen: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct CommitRowView {
    pub short_hash: String,
    pub message: String,
    pub branch: String,
    pub files_changed: i32,
    pub insertions: i32,
    pub deletions: i32,
    pub committed_at: String,
}
