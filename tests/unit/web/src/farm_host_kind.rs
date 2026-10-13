//! An agent's `host:` value is the spelling core's `HostKind` parses: every
//! host the farm accepts maps to one `HostKind`, the default is Claude Code,
//! and the schema's `CHECK` lists the same four spellings for both the agent
//! and the run row. `farm_agent_host` covers the frontmatter side.

use systemprompt::agent_runs::HostKind;
use systemprompt_web_admin_farm::services::farm_sync::{DEFAULT_HOST, HOSTS};

use crate::support::repo_root;

#[test]
fn every_farm_host_maps_to_one_core_host_kind() {
    let kinds: Vec<HostKind> = HOSTS
        .iter()
        .map(|host| host.parse::<HostKind>().expect(host))
        .collect();
    assert_eq!(
        kinds,
        vec![
            HostKind::ClaudeCode,
            HostKind::CodexCli,
            HostKind::OpenCode,
            HostKind::Hermes,
        ]
    );
    for (host, kind) in HOSTS.iter().zip(&kinds) {
        assert_eq!(kind.as_str(), *host, "the spelling round-trips");
    }
}

#[test]
fn the_default_host_is_claude_code() {
    assert_eq!(
        DEFAULT_HOST.parse::<HostKind>().expect("default parses"),
        HostKind::ClaudeCode
    );
}

#[test]
fn a_host_core_does_not_know_is_refused() {
    for bad in ["cursor", "Claude-Code", "", "claude_code"] {
        assert!(bad.parse::<HostKind>().is_err(), "{bad:?} must not parse");
    }
}

#[test]
fn the_schema_checks_the_same_four_hosts_on_agents_and_runs() {
    let schema =
        std::fs::read_to_string(repo_root().join("extensions/web/schema/52_agent_farm.sql"))
            .expect("read the farm schema");
    let quoted: Vec<String> = HOSTS.iter().map(|h| format!("'{h}'")).collect();
    let check = format!("CHECK (host IN ({}))", quoted.join(", "));
    assert_eq!(
        schema.matches(check.as_str()).count(),
        2,
        "agent_farm_agents and agent_farm_runs each carry {check}"
    );
}
