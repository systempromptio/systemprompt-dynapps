//! The runner's egress allowlist: the policy a run carries, the URL hosts it
//! adds, the address parser and the nftables ruleset the worker loads.

use std::collections::BTreeMap;
use std::net::IpAddr;
use std::time::Duration;

use systemprompt::agent_runs::{HostKind, McpServerSpec, PermissionMode, RunRequest};
use systemprompt_web_admin_farm::config::{FlyRunnerConfig, parse_farms_doc};
use systemprompt_web_admin_farm::services::runner::fly::egress::{
    EgressPolicy, FLY_PRIVATE_NET, Net, nft_ruleset, url_host,
};

fn request(mcp_url: &str) -> RunRequest {
    RunRequest {
        host: HostKind::ClaudeCode,
        prompt: "go".to_owned(),
        system_prompt_append: None,
        persona: None,
        model: None,
        allowed_tools: Vec::new(),
        disallowed_tools: Vec::new(),
        permission_mode: PermissionMode::AcceptEdits,
        permission_prompts_none: false,
        workspace: "/w".into(),
        add_dirs: Vec::new(),
        mcp_servers: BTreeMap::from([(
            "odoo".to_owned(),
            McpServerSpec::Http {
                url: mcp_url.to_owned(),
                headers: BTreeMap::new(),
            },
        )]),
        output_schema: None,
        max_turns: Some(1),
        max_budget_usd: Some(1.0),
        wall_clock: Duration::from_secs(60),
        resume_session_id: None,
        attempt: 1,
        attribution_scopes: Vec::new(),
        plugin_id: None,
        plugin_dirs: Vec::new(),
    }
}

#[test]
fn url_host_strips_scheme_port_path_and_brackets() {
    assert_eq!(
        url_host("http://app.internal:8080/api").as_deref(),
        Some("app.internal")
    );
    assert_eq!(
        url_host("https://[fdaa::5]:8080/x").as_deref(),
        Some("fdaa::5")
    );
    assert_eq!(
        url_host("https://u:p@h.example/x?q").as_deref(),
        Some("h.example")
    );
    assert_eq!(url_host("https:///nohost"), None);
}

#[test]
fn policy_adds_run_hosts_to_configured_entries() {
    let configured = vec!["api.anthropic.com".to_owned(), " 10.0.0.0/8 ".to_owned()];
    let policy = EgressPolicy::for_run(
        &configured,
        &request("http://mcp.internal:8080/api/v1/mcp/odoo/mcp"),
        &["http://app.internal:8080"],
    );
    assert_eq!(
        policy.allow,
        [
            "10.0.0.0/8",
            "api.anthropic.com",
            "app.internal",
            "mcp.internal"
        ]
    );
    let label = policy.label();
    assert!(label.starts_with("allow [lo, fdaa::/16, "), "{label}");
    assert!(label.ends_with("reject all else"), "{label}");
}

#[test]
fn net_parses_addresses_and_blocks_but_not_names() {
    let v4: IpAddr = "1.2.3.4".parse().expect("ip");
    assert_eq!(
        Net::parse("1.2.3.4"),
        Some(Net {
            addr: v4,
            prefix: 32
        })
    );
    assert_eq!(Net::parse("1.2.3.0/24").map(|n| n.prefix), Some(24));
    assert_eq!(Net::parse("fdaa::3").map(|n| n.prefix), Some(128));
    assert_eq!(Net::parse("1.2.3.4/33"), None);
    assert_eq!(Net::parse("api.anthropic.com"), None);
}

#[test]
fn ruleset_allows_lo_6pn_and_listed_nets_then_rejects() {
    let nets = [
        Net::parse("1.2.3.4").expect("v4"),
        Net::parse("2001:db8::/32").expect("v6"),
    ];
    let rules = nft_ruleset(&nets);
    assert!(rules.contains("delete table inet farm_egress"));
    assert!(rules.contains("policy drop;"));
    assert!(rules.contains("oif \"lo\" accept"));
    assert!(rules.contains(&format!("ip6 daddr {FLY_PRIVATE_NET} accept")));
    assert!(rules.contains("ip daddr 1.2.3.4/32 accept"));
    assert!(rules.contains("ip6 daddr 2001:db8::/32 accept"));
    let accept_last = rules.rfind("accept").expect("accepts");
    let reject = rules.rfind("\n    reject\n").expect("final reject");
    assert!(
        reject > accept_last,
        "reject must close the chain:\n{rules}"
    );
}

#[test]
fn egress_allow_is_read_from_the_farms_file() {
    let doc = parse_farms_doc(
        "runner:\n  mode: fly_machine\n  fly:\n    egress_allow: [api.anthropic.com, 1.2.3.4]\n",
    )
    .expect("parses");
    let fly = doc.runner.expect("runner").fly.expect("fly");
    assert_eq!(fly.egress_allow, ["api.anthropic.com", "1.2.3.4"]);
    assert!(FlyRunnerConfig::default().egress_allow.is_empty());
}
