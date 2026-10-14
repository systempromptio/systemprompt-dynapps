//! The two Dynapps portal servers (`dynapps-odoo`, `dynbrain`) are external MCP
//! servers each person connects with their own portal.dynapps.es account. They
//! work only if the YAML takes the generic per-user OAuth path end to end:
//! parsed and validated by core's own `Deployment`, resolved as a generic
//! provider, loaded by the aggregator, carried by a plugin and gated by a rule.
//! These checks pin that wiring; the OAuth round-trip itself needs the live
//! portal.

use std::collections::HashMap;

use systemprompt::models::mcp::{Deployment, McpServerType};
use systemprompt_web_admin_platform::connector_oauth::Provider;
use systemprompt_web_admin_platform::connector_oauth::generic::{same_issuer, validate_endpoint};

use crate::support::repo_root;

const SERVERS: [&str; 2] = ["dynapps-odoo", "dynbrain"];
const ISSUER: &str = "https://portal.dynapps.es";

fn read(path: &str) -> String {
    std::fs::read_to_string(repo_root().join(path))
        .unwrap_or_else(|e| panic!("{path} is readable: {e}"))
}

fn deployment(name: &str) -> Deployment {
    #[derive(serde::Deserialize)]
    struct File {
        mcp_servers: HashMap<String, Deployment>,
    }
    let path = format!("services/mcp/{name}.yaml");
    let mut file: File = serde_yaml::from_str(&read(&path))
        .unwrap_or_else(|e| panic!("{path} parses as core Deployment: {e}"));
    file.mcp_servers
        .remove(name)
        .unwrap_or_else(|| panic!("{path} declares mcp_servers.{name}"))
}

#[test]
fn each_portal_server_validates_as_an_external_generic_connector() {
    for name in SERVERS {
        let server = deployment(name);
        server
            .validate(name)
            .unwrap_or_else(|e| panic!("{name} passes core validation: {e}"));
        assert_eq!(server.server_type, McpServerType::External, "{name}");
        assert!(server.enabled, "{name} ships enabled");
        assert!(server.tool_policy.is_some(), "{name} declares tool_policy");
        assert!(
            server.external_auth.is_none(),
            "{name}: core supplies the broker only when external_auth is absent"
        );
        assert!(
            !server.oauth.required,
            "{name}: inbound access is not the portal grant"
        );
        let endpoint = server.endpoint.clone().expect("endpoint");
        assert!(
            endpoint.starts_with(&format!("{ISSUER}/mcp/")),
            "{name}: {endpoint}"
        );

        let connector = server.connector.expect("connector block");
        assert_eq!(connector.adapter, "generic", "{name}");
        assert_eq!(
            connector.scopes,
            vec!["odoo".to_owned()],
            "{name}: the issuer's only scope"
        );
        assert!(
            connector.client_id_secret.is_none() && connector.client_secret.is_none(),
            "{name}: the portal registers clients dynamically"
        );
        assert!(
            connector.authorization_origins.is_empty(),
            "{name}: every OAuth endpoint is on the resource origin"
        );
    }
}

#[test]
fn the_portal_servers_resolve_as_generic_providers() {
    for name in SERVERS {
        let provider = Provider::try_from(name.to_owned()).expect("valid connector id");
        assert_eq!(provider, Provider::Generic(name.to_owned()));
        assert_eq!(provider.slug(), name);
    }
}

#[test]
fn the_portal_issuer_endpoints_are_trusted_for_each_resource() {
    // Why: the values portal.dynapps.es advertised in its RFC 8414 metadata on
    // 2026-10-14. A move off the resource origin needs `authorization_origins`.
    let advertised = [
        "https://portal.dynapps.es/oauth/master/authorize",
        "https://portal.dynapps.es/oauth/master/token",
        "https://portal.dynapps.es/oauth/master/register",
        "https://portal.dynapps.es/.well-known/oauth-authorization-server",
    ];
    for name in SERVERS {
        let endpoint = deployment(name).endpoint.expect("endpoint");
        assert!(
            validate_endpoint(&endpoint, &[], ISSUER).is_ok(),
            "{name} issuer"
        );
        for url in advertised {
            assert!(
                validate_endpoint(&endpoint, &[], url).is_ok(),
                "{name}: {url}"
            );
        }
        assert!(validate_endpoint(&endpoint, &[], "https://evil.dynapps.es/token").is_err());
    }
    assert!(same_issuer("https://portal.dynapps.es/", ISSUER));
}

#[test]
fn the_portal_servers_are_loaded_carried_and_gated() {
    let aggregator = read("services/config/config.yaml");
    let plugin = read("services/plugins/dynapps-portal/config.yaml");
    let marketplace = read("services/marketplaces/dynapps-portal/config.yaml");
    let rules = read("services/access-control/rules.yaml");
    for name in SERVERS {
        assert!(
            aggregator.contains(&format!("../mcp/{name}.yaml")),
            "{name} included"
        );
        assert!(
            plugin.contains(&format!("- {name}\n")),
            "dynapps-portal plugin carries {name}"
        );
        assert!(
            marketplace.contains(&format!("- {name}\n")),
            "marketplace lists {name}"
        );
        assert!(
            rules.contains(&format!("entity: mcp_server/{name}\n")),
            "{name} gated"
        );
    }
    for entity in ["marketplace/dynapps-portal", "plugin/dynapps-portal"] {
        assert!(
            rules.contains(&format!("entity: {entity}\n")),
            "{entity} gated"
        );
    }
    assert!(
        !plugin.contains("hooks:"),
        "dynapps-commons is the only owner of the governance hooks"
    );
}
