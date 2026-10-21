//! The shipped chain on the hosted Odoo plane: a `dynapps-odoo` write tool
//! (`orm_write`, `orm_create`, `orm_unlink`, `orm_execute_kw`,
//! `orm_post_message`, `orm_send_mail`), under the gateway's or the bridge's
//! (`mcp__plugin_dynapps-portal_dynapps-odoo__`) naming, is flagged for
//! a plain caller and passes for `odoo_writer` or admin, its read tools pass,
//! and the role unlocks nothing outside `mcp__dynapps-odoo__`. The tool plane
//! runs in warn mode (services/governance/config.yaml), so a
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
    let user = UserId::new("odoo-farm-user");
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

const WRITES: [&str; 9] = [
    "mcp__dynapps-odoo__orm_write",
    "mcp__dynapps-odoo__orm_create",
    "mcp__dynapps-odoo__orm_unlink",
    "mcp__dynapps-odoo__orm_execute_kw",
    "mcp__dynapps-odoo__orm_post_message",
    "mcp__dynapps-odoo__orm_send_mail",
    "mcp__plugin_dynapps-portal_dynapps-odoo__orm_write",
    "mcp__plugin_dynapps-portal_dynapps-odoo__orm_post_message",
    "mcp__plugin_dynapps-portal_dynapps-odoo__orm_send_mail",
];

#[test]
fn write_tools_are_flagged_without_the_role() {
    for tool in WRITES {
        let decision = decide(tool, AccessScope::User, &["user"]);
        assert_eq!(decision.tag(), DecisionTag::Warn, "{tool}");
    }
}

#[test]
fn the_role_or_admin_lets_them_through() {
    for tool in WRITES {
        let granted = decide(tool, AccessScope::User, &["user", "odoo_writer"]);
        assert!(granted.permits(), "{tool}: {granted:?}");
        let admin = decide(tool, AccessScope::Admin, &["admin"]);
        assert!(admin.permits(), "{tool}: {admin:?}");
    }
}

#[test]
fn reads_pass_and_the_role_unlocks_nothing_elsewhere() {
    for tool in [
        "mcp__dynapps-odoo__orm_search_read",
        "mcp__dynapps-odoo__odoo_describe_model",
        "mcp__dynapps-odoo__odoo_aggregate",
        "mcp__plugin_dynapps-portal_dynapps-odoo__orm_search_read",
    ] {
        let decision = decide(tool, AccessScope::User, &["user"]);
        assert!(decision.permits(), "{tool}: {decision:?}");
    }
    let elsewhere = decide("mcp__crm__lead_delete", AccessScope::User, &["odoo_writer"]);
    assert_eq!(elsewhere.tag(), DecisionTag::Warn);
}
