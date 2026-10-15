//! The shipped chain on the Odoo plane: a destructive or erasure Odoo tool is
//! flagged for a plain caller and passes for `odoo_destructive` or admin, and
//! the role unlocks nothing outside `mcp__odoo__`. The tool plane runs in warn
//! mode (services/governance/config.yaml, 2026-10-15), so a would-be refusal
//! is recorded as `Warn`; restoring `mode: enforce` turns these back to `Deny`.

use systemprompt::identifiers::{CallId, McpToolName, SessionId, UserId};
use systemprompt_security::authz::types::{Decision, DecisionTag};
use systemprompt_security::policy::types::AccessScope;
use systemprompt_security::policy::{
    AgentScope, GovernanceConfig, GovernanceEngine, GovernedInput, GovernedTarget, McpToolInput,
    PolicyContext,
};

fn decide(tool: &str, scope: AccessScope, roles: &[&str]) -> Decision {
    let cfg = GovernanceConfig::parse(include_str!("../../../../services/governance/config.yaml"))
        .unwrap();
    let engine = GovernanceEngine::from_config(&cfg).unwrap();
    let roles: Vec<String> = roles.iter().map(|r| (*r).to_owned()).collect();
    let user = UserId::new("svc-odoo-farm");
    let session = SessionId::generate();
    let call = CallId::generate();
    let input = GovernedInput::tool_arguments(McpToolInput::new(serde_json::json!({})));
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
        })
        .decision
}

#[test]
fn destructive_odoo_tools_are_flagged_without_the_role() {
    for tool in [
        "mcp__odoo__lead_delete",
        "mcp__odoo__partner_anonymise",
        "mcp__odoo__gdpr_erase",
    ] {
        let decision = decide(tool, AccessScope::User, &["user"]);
        assert_eq!(decision.tag(), DecisionTag::Warn, "{tool}");
    }
}

#[test]
fn the_role_or_admin_lets_them_through() {
    for tool in ["mcp__odoo__lead_delete", "mcp__odoo__gdpr_erase"] {
        let granted = decide(tool, AccessScope::User, &["user", "odoo_destructive"]);
        assert!(granted.permits(), "{tool}: {granted:?}");
        let admin = decide(tool, AccessScope::Admin, &["admin"]);
        assert!(admin.permits(), "{tool}: {admin:?}");
    }
}

#[test]
fn reads_pass_and_the_role_unlocks_nothing_elsewhere() {
    assert!(decide("mcp__odoo__lead_list", AccessScope::User, &["user"]).permits());
    let elsewhere = decide(
        "mcp__crm__lead_delete",
        AccessScope::User,
        &["odoo_destructive"],
    );
    assert_eq!(elsewhere.tag(), DecisionTag::Warn);
}
