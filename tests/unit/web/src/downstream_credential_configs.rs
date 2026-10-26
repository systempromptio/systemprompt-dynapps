//! The committed portal connector configs are load-bearing and otherwise
//! unvalidated until boot: nothing else in the suite reads the external MCP
//! server YAMLs that a per-user OAuth grant fronts. These checks parse every
//! one of them the way the loaders do, so a typo'd key or a server that
//! slipped out of the aggregator fails here rather than at a user's first
//! tool call.

use crate::support::repo_root;

const PORTAL_SERVERS: [&str; 2] = ["dynapps-odoo", "dynbrain"];

fn server(name: &str) -> serde_yaml::Value {
    let path = repo_root()
        .join("services/mcp")
        .join(format!("{name}.yaml"));
    let yaml = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{} is readable: {e}", path.display()));
    let doc: serde_yaml::Value = serde_yaml::from_str(&yaml)
        .unwrap_or_else(|e| panic!("{} parses as YAML: {e}", path.display()));
    doc.get("mcp_servers")
        .and_then(|s| s.get(name))
        .unwrap_or_else(|| panic!("{name}.yaml declares an `mcp_servers.{name}` entry"))
        .clone()
}

#[test]
fn every_portal_server_is_an_enabled_generic_oauth_connector() {
    for name in PORTAL_SERVERS {
        let entry = server(name);
        assert_eq!(
            entry["enabled"].as_bool(),
            Some(true),
            "{name} ships enabled"
        );
        assert_eq!(
            entry["server_type"].as_str(),
            Some("external"),
            "{name} is an external server"
        );
        assert_eq!(
            entry["connector"]["adapter"].as_str(),
            Some("generic"),
            "{name} connects through the generic OAuth connector"
        );
        // Why: the per-user connector grant governs the upstream call;
        // `oauth.required: true` alongside it would gate the same call twice
        // against two different credentials.
        assert_eq!(
            entry["oauth"]["required"].as_bool(),
            Some(false),
            "{name}: the connector governs auth, so oauth.required must be false"
        );
        assert!(
            entry["endpoint"]
                .as_str()
                .is_some_and(|e| e.starts_with("https://portal.dynapps.es/")),
            "{name} is served by the Dynapps portal"
        );
    }
}

#[test]
fn the_portal_servers_are_listed_in_the_aggregator() {
    // Why: a flat resource file that is not in `includes:` is never loaded, and
    // nothing reports it — the server simply does not exist.
    let aggregator = std::fs::read_to_string(repo_root().join("services/config/config.yaml"))
        .expect("services/config/config.yaml is readable");
    for name in PORTAL_SERVERS {
        assert!(
            aggregator.contains(&format!("../mcp/{name}.yaml")),
            "services/config/config.yaml must include ../mcp/{name}.yaml"
        );
    }
}
