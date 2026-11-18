//! The shipped chain on the internal Microsoft 365 server. A tool that sends,
//! replies, forwards or deletes as the user, `d365_batch` (whose requests may
//! carry any of those), the ones that move data out without sending
//! (`d365_create_mail_rule`, `d365_share_file`,
//! `d365_upload_attachment_chunk`), the event tools that make Exchange email
//! attendees or the organizer (create, update, accept, decline, tentatively
//! accept), `d365_move_mail` (to `deleteditems` it is a soft delete) and
//! `d365_add_planner_roster_member` (it grants access) are flagged for a plain
//! caller and pass for a `knowledge_worker` or admin; every read tool of the
//! pinned Belgian surface passes for everyone, the ones whose names resemble a
//! gated tool included; the role unlocks nothing outside
//! `mcp__microsoft-365__` and the dynapps-odoo writes stay with `odoo_writer`.
//! The tool plane runs in warn mode (services/governance/config.yaml), so a
//! would-be refusal is recorded as `Warn`; restoring `mode: enforce` turns
//! these back to `Deny`.

use systemprompt::governance::authz::types::{Decision, DecisionTag};
use systemprompt::governance::policy::types::AccessScope;
use systemprompt::governance::policy::{
    AgentScope, GovernanceConfig, GovernanceEngine, GovernedInput, GovernedTarget, McpToolInput,
    PolicyContext, RateCounts,
};
use systemprompt::identifiers::{CallId, McpToolName, PolicyId, SessionId, UserId};

fn decide(tool: &str, scope: AccessScope, roles: &[&str]) -> Decision {
    let cfg = GovernanceConfig::parse(include_str!("../../../../services/governance/config.yaml"))
        .unwrap();
    let engine =
        GovernanceEngine::from_config(&cfg, Some(crate::support::uncharged_rate_windows()))
            .unwrap();
    let roles: Vec<String> = roles.iter().map(|r| (*r).to_owned()).collect();
    let user = UserId::new("m365-gate-user");
    let session = SessionId::generate();
    let call = CallId::generate();
    let input = GovernedInput::tool_arguments(McpToolInput::new(serde_json::json!({})));
    let rate_counts = RateCounts::charged([(PolicyId::new("rate_limit"), 1)]);
    engine
        .evaluate(&PolicyContext {
            target: GovernedTarget::Tool {
                tool: McpToolName::try_new(tool).unwrap(),
            },
            agent_scope: AgentScope::User {
                user_id: user.clone(),
            },
            access_scope: scope,
            roles: &roles,
            user_id: &user,
            session_id: &session,
            call_id: &call,
            input: &input,
            rate_counts: &rate_counts,
        })
        .decision
}

const GATED: [&str; 20] = [
    "mcp__microsoft-365__d365_send_mail",
    "mcp__microsoft-365__d365_send_draft",
    "mcp__microsoft-365__d365_send_chat_message",
    "mcp__microsoft-365__d365_send_channel_message",
    "mcp__microsoft-365__d365_reply_mail",
    "mcp__microsoft-365__d365_reply_to_channel_message",
    "mcp__microsoft-365__d365_forward_mail",
    "mcp__microsoft-365__d365_delete_mail",
    "mcp__microsoft-365__d365_delete_file",
    "mcp__microsoft-365__d365_batch",
    "mcp__microsoft-365__d365_create_mail_rule",
    "mcp__microsoft-365__d365_share_file",
    "mcp__microsoft-365__d365_create_event",
    "mcp__microsoft-365__d365_update_event",
    "mcp__microsoft-365__d365_accept_event",
    "mcp__microsoft-365__d365_decline_event",
    "mcp__microsoft-365__d365_tentatively_accept_event",
    "mcp__microsoft-365__d365_upload_attachment_chunk",
    "mcp__microsoft-365__d365_move_mail",
    "mcp__microsoft-365__d365_add_planner_roster_member",
];

// Read tools whose names share a word with a gated one: a `contains:` entry
// that grew too short would catch them first.
const LOOKALIKE_READS: [&str; 12] = [
    "mcp__microsoft-365__d365_list_mail",
    "mcp__microsoft-365__d365_get_mail_attachment",
    "mcp__microsoft-365__d365_list_mail_attachments",
    "mcp__microsoft-365__d365_list_calendar_view",
    "mcp__microsoft-365__d365_list_events",
    "mcp__microsoft-365__d365_get_event",
    "mcp__microsoft-365__d365_list_shared_calendar_events",
    "mcp__microsoft-365__d365_list_planner_roster_members",
    "mcp__microsoft-365__d365_download_file",
    "mcp__microsoft-365__d365_list_mail_rules",
    "mcp__microsoft-365__d365_list_file_permissions",
    "mcp__microsoft-365__d365_list_channel_message_replies",
];

#[test]
fn send_delete_and_batch_are_flagged_without_the_role() {
    for tool in GATED {
        let decision = decide(tool, AccessScope::User, &["user"]);
        assert_eq!(decision.tag(), DecisionTag::Warn, "{tool}");
    }
}

#[test]
fn a_knowledge_worker_or_admin_lets_them_through() {
    for tool in GATED {
        let granted = decide(tool, AccessScope::User, &["user", "knowledge_worker"]);
        assert!(granted.permits(), "{tool}: {granted:?}");
        let admin = decide(tool, AccessScope::Admin, &["admin"]);
        assert!(admin.permits(), "{tool}: {admin:?}");
    }
}

#[test]
fn reads_pass_and_the_role_unlocks_nothing_elsewhere() {
    for tool in LOOKALIKE_READS {
        let decision = decide(tool, AccessScope::User, &["user"]);
        assert!(decision.permits(), "{tool}: {decision:?}");
    }
    let odoo = decide(
        "mcp__dynapps-odoo__orm_unlink",
        AccessScope::User,
        &["knowledge_worker"],
    );
    assert_eq!(odoo.tag(), DecisionTag::Warn);
    let elsewhere = decide(
        "mcp__crm__lead_delete",
        AccessScope::User,
        &["knowledge_worker"],
    );
    assert_eq!(elsewhere.tag(), DecisionTag::Warn);
}

#[test]
fn every_read_tool_of_the_pinned_surface_passes_without_the_role() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../mcp/fixtures/m365_tools.json")).unwrap();
    let reads: Vec<&str> = fixture["tools"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|tool| tool["readOnlyHint"] == true)
        .filter_map(|tool| tool["name"].as_str())
        .collect();
    assert!(reads.len() > 70, "{} read tools pinned", reads.len());
    for name in reads {
        let tool = format!("mcp__microsoft-365__{name}");
        let decision = decide(&tool, AccessScope::User, &["user"]);
        assert!(decision.permits(), "{tool}: {decision:?}");
    }
}
