//! Template context types for the user pages.

use serde::Serialize;
use systemprompt::identifiers::{ContextId, UserId};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UserMarketplaceRef {
    pub id: String,
    pub name: String,
    pub source: &'static str,
}

// Why: one checkbox on the detail page. `source` carries where the membership
// came from, because a directory-sourced one is re-projected at every sign-in
// and unchecking it here would silently come back — so the control is rendered
// disabled rather than offered and then undone.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct MembershipChoiceView {
    pub id: String,
    pub name: String,
    pub held: bool,
    pub source: &'static str,
    pub from_directory: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RoleChoiceView {
    pub id: String,
    pub label: String,
    pub held: bool,
    pub platform_only: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GroupBucket {
    pub group_id: String,
    pub group_name: String,
    pub users: Vec<EnrichedUserView>,
    pub user_count: usize,
    pub total_tokens: i64,
    pub total_sessions: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EnrichedUserView {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub roles: Vec<String>,
    pub is_active: bool,
    pub last_active: String,
    pub total_events: i64,
    pub last_tool: Option<String>,
    pub custom_skills_count: i64,
    pub preferred_client: Option<String>,
    pub prompts: i64,
    pub sessions: i64,
    pub bytes: i64,
    pub logins: i64,
    #[serde(default)]
    pub group_ids: Vec<String>,
    #[serde(default)]
    pub project_ids: Vec<String>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub marketplaces: Vec<UserMarketplaceRef>,
    #[serde(default)]
    pub assigned_skills_count: i64,
    #[serde(default)]
    pub devices_count: i64,
    #[serde(default)]
    pub connected_agents: i64,
    #[serde(default)]
    pub total_agents: i64,
    #[serde(default)]
    pub lifetime_tokens: i64,
    #[serde(default)]
    pub device_freshness: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PageStatView {
    pub value: i64,
    pub label: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UsersPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub groups: Vec<GroupBucket>,
    pub scope_filter: crate::handlers::ssr::list_view::ScopeFilterView,
    // Why: the roster renders group and project *names* on chips but buckets
    // and filters by id, so both maps travel with the page rather than being
    // denormalised onto every row.
    #[serde(default)]
    pub group_names: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub project_names: std::collections::BTreeMap<String, String>,
    pub total_users: usize,
    pub active_users: usize,
    pub total_events: i64,
    #[serde(default)]
    pub page_stats: Vec<PageStatView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UserDetailPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub user: Option<crate::types::UserDetail>,
    pub gamification: Option<crate::types::UserGamificationProfile>,
    pub not_found: bool,
    // Why: read-only — group and project membership is re-projected from the
    // directory at every sign-in, so there is no control to change it here.
    #[serde(default)]
    pub user_groups: String,
    #[serde(default)]
    pub user_projects: String,
    #[serde(default)]
    pub user_adfs_groups: Vec<String>,
    #[serde(default)]
    pub group_choices: Vec<MembershipChoiceView>,
    #[serde(default)]
    pub project_choices: Vec<MembershipChoiceView>,
    #[serde(default)]
    pub role_choices: Vec<RoleChoiceView>,
    #[serde(default)]
    pub can_edit_membership: bool,
    #[serde(default)]
    pub is_self: bool,
    #[serde(default)]
    pub user_assignments: UserAssignmentSummary,
    #[serde(default)]
    pub user_devices: Vec<UserDeviceView>,
    #[serde(default)]
    pub user_devices_count: i64,
    pub runtime: Option<UserRuntimeView>,
    #[serde(default)]
    pub ai_usage: Option<UserAiUsageView>,
    #[serde(default)]
    pub conversations: Vec<UserConversationView>,
    #[serde(default)]
    pub has_conversations: bool,
    #[serde(default)]
    pub conversations_url: String,
    #[serde(default)]
    pub effective_permissions:
        Option<crate::repositories::governance::effective::EffectivePermissions>,
    #[serde(default)]
    pub has_effective_permissions: bool,
}

// Why: the gateway-backed "AI requests" block on the user-detail page.
// Why: Sourced from `ai_requests`, which is where `/v1/messages` traffic
// Why: lands — the bridge runtime block below counts hook telemetry and
// Why: reports nothing at all for a user who only ever talks to the gateway.
#[derive(Debug, Clone, Default, Serialize)]
pub(crate) struct UserAiUsageView {
    pub requests: i64,
    pub conversations: i64,
    pub failed: i64,
    pub tokens: i64,
    pub tokens_display: String,
    pub cost_display: String,
    pub first_request_at: Option<String>,
    pub last_request_at: Option<String>,
    pub top_models: Vec<UserModelShareView>,
    pub has_top_models: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UserModelShareView {
    pub model: String,
    pub requests: i64,
    pub tokens_display: String,
    pub cost_display: String,
}

// Why: one gateway conversation on the user-detail page, grouped by
// Why: `context_id`.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct UserConversationView {
    pub context_id: ContextId,
    pub context_id_short: String,
    pub title: String,
    pub summary: Option<String>,
    pub model: Option<String>,
    pub request_count: i64,
    pub message_count: i64,
    pub error_count: i64,
    pub has_errors: bool,
    pub cost_display: String,
    pub first_seen_local: Option<String>,
    pub last_seen_local: Option<String>,
    pub context_url: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub(crate) struct UserRuntimeView {
    pub connected_agents: i64,
    pub total_agents: i64,
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub last_bridge_version: Option<String>,
    pub last_os: Option<String>,
    pub last_hostname: Option<String>,
    pub last_heartbeat_at: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub(crate) struct UserAssignmentSummary {
    pub skills_count: i64,
    pub marketplaces_count: i64,
    #[serde(default)]
    pub marketplaces: Vec<UserMarketplaceRef>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UserDeviceView {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub platform: Option<String>,
    pub app_version: Option<String>,
    pub last_seen_at: Option<chrono::DateTime<chrono::Utc>>,
    pub revoked: bool,
}
