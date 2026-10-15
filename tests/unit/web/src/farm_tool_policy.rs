//! A farm's `tool_policy:` in agent-farms.yaml (the MCP server allowlist and
//! the tools every run is denied), and the shipped config: one client, one
//! nightly audit farm that runs as its launcher, reaches only the two hosted
//! portal servers and can never write to Odoo.

use systemprompt_web_admin_farm::config::{FarmToolPolicy, RunAsMode, parse_farms_doc};
use systemprompt_web_admin_farm::services::runner::tool_policy::{
    disallowed_tools, with_farm_denials,
};
use systemprompt_web_admin_farm::types::TriggeredVia;

const DOC: &str = "farms:
  - id: audit
    name: Audit
    marketplace: dynapps
    tool_policy:
      mcp_servers: [dynapps-odoo, dynbrain]
      deny: [mcp__dynapps-odoo__orm_write]
    source:
      github: dynapps/agent-farm
";

fn tools(names: &[&str]) -> Vec<String> {
    names.iter().map(|n| (*n).to_owned()).collect()
}

#[test]
fn a_tool_policy_parses_and_validates() {
    let doc = parse_farms_doc(DOC).expect("parses");
    doc.validate().expect("valid");
    let policy = &doc.farms[0].tool_policy;
    assert_eq!(policy.mcp_servers, ["dynapps-odoo", "dynbrain"]);
    assert_eq!(policy.deny, ["mcp__dynapps-odoo__orm_write"]);
}

#[test]
fn a_bad_server_or_tool_is_refused() {
    for text in [
        DOC.replace("[dynapps-odoo, dynbrain]", "[\"dyn brain\"]"),
        DOC.replace("[mcp__dynapps-odoo__orm_write]", "[Hammer]"),
        DOC.replace("deny:", "allow:"),
    ] {
        let refused = parse_farms_doc(&text).and_then(|doc| doc.validate());
        assert!(refused.is_err(), "{text}");
    }
}

#[test]
fn the_allowlist_names_the_first_server_outside_it() {
    let policy = FarmToolPolicy {
        mcp_servers: tools(&["dynapps-odoo", "dynbrain"]),
        deny: Vec::new(),
    };
    let inside = tools(&["Read", "mcp__dynapps-odoo", "mcp__dynbrain__search"]);
    assert_eq!(policy.disallowed_server(&inside), None);
    let outside = tools(&["mcp__dynapps-odoo", "mcp__systemprompt__run"]);
    assert_eq!(policy.disallowed_server(&outside), Some("systemprompt"));
    let open = FarmToolPolicy::default();
    assert_eq!(open.disallowed_server(&outside), None);
}

#[test]
fn farm_denials_are_appended_once() {
    let base = disallowed_tools(&tools(&["Read", "Write"]));
    assert!(base.iter().any(|t| t == "Bash"));
    let deny = tools(&["mcp__dynapps-odoo__orm_write", "Bash"]);
    let merged = with_farm_denials(base.clone(), &deny);
    assert_eq!(merged.len(), base.len() + 1);
    assert!(merged.iter().any(|t| t == "mcp__dynapps-odoo__orm_write"));
}

#[test]
fn the_shipped_config_is_the_one_nightly_audit_flow() {
    let text = include_str!("../../../../services/web/config/agent-farms.yaml");
    let doc = parse_farms_doc(text).expect("parses");
    doc.validate().expect("valid");
    assert_eq!(doc.clients.len(), 1);
    assert_eq!(doc.clients[0].slug, "dynapps");
    assert_eq!(doc.farms.len(), 1);
    let farm = &doc.farms[0];
    assert_eq!(farm.id, "dynapps-night-audit");
    assert_eq!(farm.run_as.as_ref().map(|r| r.mode), Some(RunAsMode::User));
    assert_eq!(farm.tool_policy.mcp_servers, ["dynapps-odoo", "dynbrain"]);
    for write in ["orm_create", "orm_write", "orm_unlink", "orm_execute_kw"] {
        let tool = format!("mcp__dynapps-odoo__{write}");
        assert!(
            farm.tool_policy.deny.contains(&tool),
            "{tool} is not denied"
        );
    }
    assert_eq!(farm.defaults.model.as_deref(), Some("claude-opus-5-5"));
    assert_eq!(farm.defaults.max_turns, 40);
    assert!((farm.defaults.max_budget_usd - 8.0).abs() < f64::EPSILON);
}

#[test]
fn the_scheduler_queues_the_audit_nightly() {
    let text = include_str!("../../../../services/scheduler/config.yaml");
    let entry = text
        .split("- name: farm_scheduled_run")
        .nth(1)
        .and_then(|rest| rest.split("\n\n").next())
        .expect("farm_scheduled_run entry");
    assert!(entry.contains("schedule: \"0 0 2 * * *\""), "{entry}");
    assert!(entry.contains("farm: dynapps-night-audit"), "{entry}");
    assert!(entry.contains("agent: accounting_night_audit"), "{entry}");
    assert!(entry.contains("launcher: ed@systemprompt.io"), "{entry}");
    assert!(entry.contains("case: nightly_previous_month"), "{entry}");
    assert_eq!(
        TriggeredVia::parse("scheduler"),
        Some(TriggeredVia::Scheduler)
    );
}
