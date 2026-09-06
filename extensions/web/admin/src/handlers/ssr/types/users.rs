//! Template context types for the user pages.

use serde::Serialize;
use systemprompt::identifiers::ContextId;

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
