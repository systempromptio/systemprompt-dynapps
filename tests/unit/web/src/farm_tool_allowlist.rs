//! A farm agent's declared tools as the run enforces them: declared tools
//! pass, an undeclared MCP tool is refused under `farm_tool_allowlist`, and
//! built-in tools keep the `--disallowedTools` treatment they had.

use systemprompt_web_admin_core::util::tool_allowlist::{
    FARM_TOOL_ALLOWLIST_POLICY, tool_permitted, undeclared_mcp_tool,
};
use systemprompt_web_admin_farm::services::runner::tool_policy::{allowed_tools, disallowed_tools};

fn tools(list: &[&str]) -> Vec<String> {
    list.iter().map(|t| (*t).to_owned()).collect()
}

fn triage() -> Vec<String> {
    allowed_tools(&tools(&[
        "Read",
        "Write",
        "mcp__dynapps-odoo__crm_lead_search",
        "mcp__dynapps-odoo__crm_lead_update",
    ]))
}

#[test]
fn policy_id_is_stable() {
    assert_eq!(FARM_TOOL_ALLOWLIST_POLICY, "farm_tool_allowlist");
}

#[test]
fn declared_mcp_tool_is_allowed() {
    let allow = triage();
    assert!(tool_permitted("mcp__dynapps-odoo__crm_lead_search", &allow));
    assert!(!undeclared_mcp_tool("mcp__dynapps-odoo__crm_lead_update", &allow));
}

#[test]
fn undeclared_mcp_tool_is_denied() {
    let allow = triage();
    assert!(undeclared_mcp_tool("mcp__dynapps-odoo__partner_delete", &allow));
    assert!(undeclared_mcp_tool("mcp__github__create_issue", &allow));
    assert!(undeclared_mcp_tool(
        "mcp__dynapps-odoo__crm_lead_search_all",
        &allow
    ));
}

#[test]
fn server_and_wildcard_entries_cover_their_server_only() {
    let server = tools(&["mcp__dynapps-odoo"]);
    assert!(!undeclared_mcp_tool("mcp__dynapps-odoo__partner_get", &server));
    assert!(undeclared_mcp_tool("mcp__dynapps-odoo_admin__partner_get", &server));
    let wildcard = tools(&["mcp__dynapps-odoo__*"]);
    assert!(!undeclared_mcp_tool("mcp__dynapps-odoo__partner_get", &wildcard));
    assert!(undeclared_mcp_tool("mcp__dynbrain__docs_search", &wildcard));
}

#[test]
fn builtin_tools_are_not_the_hooks_concern() {
    let allow = triage();
    assert!(!undeclared_mcp_tool("Bash", &allow));
    assert!(!undeclared_mcp_tool("TodoWrite", &allow));
    let denied = disallowed_tools(&tools(&["Read", "Write", "mcp__dynapps-odoo__note_add"]));
    assert!(denied.iter().any(|t| t == "Bash"));
    assert!(!denied.iter().any(|t| t == "Read" || t == "Write"));
    assert!(!denied.iter().any(|t| t.starts_with("mcp__")));
}

#[test]
fn skill_tool_is_always_permitted() {
    let allow = triage();
    assert!(allow.iter().any(|t| t == "Skill"));
    assert!(tool_permitted("Skill", &tools(&["Read"])));
}

#[test]
fn an_agent_without_a_tools_list_is_unrestricted() {
    assert!(!undeclared_mcp_tool("mcp__dynapps-odoo__partner_delete", &[]));
    assert!(disallowed_tools(&[]).is_empty());
}
