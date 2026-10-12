//! The optional `host:` agent frontmatter key: absent means `claude-code`,
//! each of the four hosts parses, anything else is a sync error.

use systemprompt_web_admin_farm::services::farm_sync::{DEFAULT_HOST, HOSTS, parse_agent_markdown};

const AGENT: &str = "---\nname: abc_agent\ndescription: d\ntools: [Read]\nplugin: p\n\
entry_prompt: \"go {{fixture_path}}\"\nfixture: fix_one\n---\nBody\n";

const PATH: &str = "agents/abc_agent.md";

fn with_host(host: &str) -> String {
    AGENT.replace("plugin: p\n", &format!("plugin: p\nhost: {host}\n"))
}

#[test]
fn host_defaults_to_claude_code() {
    let agent = parse_agent_markdown(PATH, AGENT).expect("parses");
    assert_eq!(agent.host, DEFAULT_HOST);
    assert_eq!(agent.host, "claude-code");
}

#[test]
fn every_valid_host_parses() {
    assert_eq!(HOSTS, ["claude-code", "codex-cli", "opencode", "hermes"]);
    for host in HOSTS {
        let agent = parse_agent_markdown(PATH, &with_host(host)).expect(host);
        assert_eq!(agent.host, host);
    }
}

#[test]
fn unknown_host_names_agent_and_value() {
    let err = parse_agent_markdown(PATH, &with_host("cursor"))
        .expect_err("cursor is not a host")
        .to_string();
    assert!(err.contains("abc_agent"), "{err}");
    assert!(err.contains("cursor"), "{err}");
}
