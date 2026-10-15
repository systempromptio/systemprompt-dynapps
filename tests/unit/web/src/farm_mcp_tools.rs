//! `mcp__<server>[__<tool>]` entries in an agent's `tools:`: the shapes the
//! parser accepts, the ones it refuses, and the HTTP server specs a run is
//! handed for them, bearer added only at claim time.

use systemprompt::agent_runs::McpServerSpec;
use systemprompt_web_admin_farm::services::farm_sync::parse_agent_markdown;
use systemprompt_web_admin_farm::services::farm_sync::validate::{is_allowed_tool, mcp_server_of};
use systemprompt_web_admin_farm::services::runner::prepare::mcp_servers;

const AGENT: &str = "---\nname: abc_agent\ndescription: d\ntools: [Read]\nplugin: p\n\
fixture: fix_one\nentry_prompt: \"Work in {{fixture_path}}\"\n---\nBody\n";

#[test]
fn mcp_entries_name_their_server() {
    assert_eq!(mcp_server_of("mcp__dynapps-odoo"), Some("dynapps-odoo"));
    assert_eq!(mcp_server_of("mcp__dynapps-odoo__lead_list"), Some("dynapps-odoo"));
    assert_eq!(mcp_server_of("mcp__agent-farm__*"), Some("agent-farm"));
    assert_eq!(mcp_server_of("Read"), None);
    for bad in [
        "mcp__",
        "mcp____tool",
        "mcp__dynapps-odoo__",
        "mcp__od oo",
        "mcp__dynapps-odoo__a/b",
    ] {
        assert_eq!(mcp_server_of(bad), None, "{bad}");
        assert!(!is_allowed_tool(bad), "{bad}");
    }
}

#[test]
fn an_agent_may_declare_mcp_tools() {
    let text = AGENT.replace(
        "tools: [Read]",
        "tools: [Write, mcp__dynapps-odoo__lead_list, mcp__dynapps-odoo]",
    );
    let agent = parse_agent_markdown("agents/abc_agent.md", &text).expect("parses");
    assert_eq!(agent.tools.len(), 3);
    let bad = AGENT.replace("tools: [Read]", "tools: [mcp__dynapps-odoo__]");
    assert!(parse_agent_markdown("agents/abc_agent.md", &bad).is_err());
}

#[test]
fn each_referenced_server_becomes_one_http_spec() {
    let tools = [
        "Write".to_owned(),
        "mcp__dynapps-odoo__lead_list".to_owned(),
        "mcp__dynapps-odoo__lead_delete".to_owned(),
    ];
    let servers = mcp_servers(&tools, "http://127.0.0.1:8080");
    assert_eq!(servers.len(), 1);
    match &servers["dynapps-odoo"] {
        McpServerSpec::Http { url, headers } => {
            assert_eq!(url, "http://127.0.0.1:8080/api/v1/mcp/dynapps-odoo/mcp");
            assert!(headers.is_empty(), "the stored request carries no bearer");
        },
        McpServerSpec::Stdio { .. } => panic!("expected an HTTP server"),
    }
}
