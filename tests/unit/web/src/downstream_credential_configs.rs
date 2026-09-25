//! The committed downstream-credential configs are load-bearing and otherwise
//! unvalidated until boot: nothing else in the suite reads
//! `services/web/config/salesforce.yaml` or the external MCP server YAMLs that
//! depend on a per-user accessor. These checks parse every one of them the way
//! the loaders do, so a typo'd key or an accessor path that disagrees with the
//! route it names fails here rather than at a user's first tool call.
//!
//! They also pin the posture: the Salesforce app ships disabled ahead of its
//! credentials, the hosted Atlassian connector is the only Atlassian server,
//! and the retired v1 `jira`/`confluence` servers cannot creep back.

use std::path::PathBuf;

use systemprompt_web_admin::SalesforceConfig;

use crate::support::repo_root;

// Why: must match the routes registered by `connector_api_router` /
// `salesforce_api_router` under the `/api/public` nest. Core resolves the
// accessor as `api_external_url + token_endpoint`, so a disagreement here is a
// 404 at tool-call time and nowhere earlier.
const ATLASSIAN_ACCESSOR: &str = "/api/public/connectors/atlassian/token";
const SALESFORCE_SERVERS: [&str; 1] = ["salesforce"];
// Servers that trust the identity this instance signs rather than a banked
// provider token; their accessor is `identity_token::accessor_path`.
const IDENTITY_SERVERS: [&str; 1] = ["sfcore_dev"];

fn web_config(name: &str) -> PathBuf {
    repo_root().join("services/web/config").join(name)
}

fn mcp_config(name: &str) -> serde_yaml::Value {
    let path = repo_root().join("services/mcp").join(name);
    let yaml = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{} is readable: {e}", path.display()));
    serde_yaml::from_str(&yaml).unwrap_or_else(|e| panic!("{} parses as YAML: {e}", path.display()))
}

fn server(file: &str, name: &str) -> serde_yaml::Value {
    mcp_config(file)
        .get("mcp_servers")
        .and_then(|s| s.get(name))
        .unwrap_or_else(|| panic!("{file} declares an `mcp_servers.{name}` entry"))
        .clone()
}

#[test]
fn the_salesforce_config_parses_as_the_loader_reads_it() {
    let path = web_config("salesforce.yaml");
    let yaml = std::fs::read_to_string(&path).expect("salesforce.yaml is readable");
    let config: SalesforceConfig = serde_yaml::from_str(&yaml)
        .unwrap_or_else(|e| panic!("{} parses as SalesforceConfig: {e}", path.display()));

    assert!(
        config.my_domain.starts_with("https://"),
        "my_domain doubles as the JWT-bearer audience and must be the https My Domain base, got {}",
        config.my_domain
    );
}

#[test]
fn the_salesforce_app_ships_disabled_and_its_server_enabled() {
    // Why: the app needs a credential that does not exist in any environment
    // yet; the server is visible only to linked users, so it can stay on.
    for name in SALESFORCE_SERVERS {
        assert_eq!(
            server(&format!("{name}.yaml"), name)["enabled"].as_bool(),
            Some(true),
            "{name} must ship enabled"
        );
    }
    let yaml = std::fs::read_to_string(web_config("salesforce.yaml")).expect("readable");
    let value: serde_yaml::Value = serde_yaml::from_str(&yaml).expect("parses");
    assert_eq!(
        value.get("enabled").and_then(serde_yaml::Value::as_bool),
        Some(false),
        "salesforce.yaml must ship disabled"
    );
}

#[test]
fn every_external_server_points_at_the_accessor_that_serves_it() {
    let salesforce = SALESFORCE_SERVERS.map(|name| {
        (
            format!("{name}.yaml"),
            name,
            format!("/api/public/connectors/{name}/token"),
        )
    });
    let mut servers = vec![(
        "atlassian.yaml".to_owned(),
        "atlassian",
        ATLASSIAN_ACCESSOR.to_owned(),
    )];
    servers.extend(salesforce);
    servers.extend(IDENTITY_SERVERS.map(|name| {
        (
            format!("{name}.yaml"),
            name,
            systemprompt_web_admin::identity_token::accessor_path(name),
        )
    }));
    for (file, name, accessor) in servers {
        let entry = server(&file, name);
        let external_auth = entry
            .get("external_auth")
            .unwrap_or_else(|| panic!("{file}: {name} must declare external_auth"));

        assert_eq!(
            external_auth
                .get("token_endpoint")
                .and_then(serde_yaml::Value::as_str),
            Some(accessor.as_str()),
            "{file}: token_endpoint must name the route that actually serves it"
        );

        // Why: the presence of `external_auth` is what switches core off
        // forwarding our own JWT. `oauth.required: true` alongside it would gate
        // the same call twice against two different credentials.
        assert_eq!(
            entry
                .get("oauth")
                .and_then(|o| o.get("required"))
                .and_then(serde_yaml::Value::as_bool),
            Some(false),
            "{file}: external_auth governs auth, so oauth.required must be false"
        );

        assert_eq!(
            entry.get("type").and_then(serde_yaml::Value::as_str),
            Some("external"),
            "{file}: a server with external_auth must be type: external"
        );
    }
}

#[test]
fn the_google_workspace_server_leaves_its_accessor_to_the_generic_connector() {
    // Why: with a `connector:` block and no `external_auth`, core synthesises
    // the accessor route itself; declaring both would name the route twice and
    // let them drift apart.
    {
        let (file, name) = ("google_workspace.yaml", "google_workspace");
        let entry = server(file, name);
        assert!(
            entry.get("external_auth").is_none(),
            "{file}: the generic connector owns the accessor"
        );
        let connector = entry
            .get("connector")
            .unwrap_or_else(|| panic!("{file}: {name} must declare a connector block"));
        assert_eq!(
            connector.get("adapter").and_then(serde_yaml::Value::as_str),
            Some("generic"),
            "{file}: adapter"
        );
        // Why: Google issues a refresh token only when offline access is
        // requested; without it the connection expires with the hour-long
        // access token.
        assert_eq!(
            connector
                .get("authorization_params")
                .and_then(|p| p.get("access_type"))
                .and_then(serde_yaml::Value::as_str),
            Some("offline"),
            "{file}: access_type=offline is what keeps the grant alive"
        );
        assert_eq!(
            connector
                .get("identity")
                .and_then(serde_yaml::Value::as_str),
            Some("userinfo"),
            "{file}: identity"
        );
        assert_eq!(
            entry
                .get("oauth")
                .and_then(|o| o.get("required"))
                .and_then(serde_yaml::Value::as_bool),
            Some(false),
            "{file}: oauth.required"
        );
        assert_eq!(
            entry.get("type").and_then(serde_yaml::Value::as_str),
            Some("external"),
            "{file}: type"
        );
        assert_eq!(
            entry.get("tool_policy").and_then(serde_yaml::Value::as_str),
            Some("allow"),
            "{file}: a server without tool_policy is withheld from the bridge"
        );
    }
}

#[test]
fn the_google_workspace_server_targets_agent_search_with_its_scope() {
    // Why: the MCP endpoint is regional and must match the engine's location
    // — Astound's workspace-search engine is `global`, so the host is the
    // global one — and its protected-resource metadata names exactly this
    // scope; a token minted without it is accepted by Google's OAuth server and
    // then rejected by the MCP endpoint, which surfaces as a failed Test on the
    // Connectors page and nowhere earlier.
    let entry = server("google_workspace.yaml", "google_workspace");
    assert_eq!(
        entry.get("endpoint").and_then(serde_yaml::Value::as_str),
        Some("https://discoveryengine.googleapis.com/mcp")
    );
    let scopes: Vec<&str> = entry
        .get("connector")
        .and_then(|c| c.get("scopes"))
        .and_then(serde_yaml::Value::as_sequence)
        .expect("google_workspace.yaml: connector.scopes is a list")
        .iter()
        .filter_map(serde_yaml::Value::as_str)
        .collect();
    assert!(
        scopes.contains(&"https://www.googleapis.com/auth/discoveryengine.readwrite"),
        "google_workspace.yaml: scopes must carry discoveryengine.readwrite, got {scopes:?}"
    );
}

#[test]
fn the_atlassian_server_uses_the_v2_streamable_http_endpoint() {
    // Why: core's MCP client speaks only StreamableHttp — it has no HTTP+SSE
    // transport — and Atlassian's /v1 forms are retired with the v1 link.
    let endpoint = server("atlassian.yaml", "atlassian")
        .get("endpoint")
        .and_then(serde_yaml::Value::as_str)
        .expect("atlassian.yaml: atlassian must declare an endpoint")
        .to_owned();
    assert!(
        endpoint.starts_with("https://mcp.atlassian.com/v2/mcp"),
        "atlassian.yaml: the hosted connector is the v2 endpoint, got {endpoint}"
    );
}

#[test]
fn the_retired_v1_atlassian_files_do_not_exist() {
    // Why: the per-server jira/confluence entries and the 3LO app config were
    // deleted with their code (2026-09-08). A file reappearing would be loaded
    // by the aggregator or the link-gate loader and fail at boot, or worse,
    // parse and gate nothing.
    for path in [
        "services/mcp/jira.yaml",
        "services/mcp/confluence.yaml",
        "services/web/config/atlassian.yaml",
        "services/access-control/atlassian.yaml",
    ] {
        assert!(
            !repo_root().join(path).exists(),
            "{path} belongs to the retired v1 Atlassian link"
        );
    }
}

#[test]
fn the_external_mcp_servers_are_listed_in_the_aggregator() {
    // Why: a flat resource file that is not in `includes:` is never loaded, and
    // nothing reports it — the server simply does not exist.
    let aggregator = std::fs::read_to_string(repo_root().join("services/config/config.yaml"))
        .expect("services/config/config.yaml is readable");
    for name in ["atlassian"].into_iter().chain(SALESFORCE_SERVERS) {
        assert!(
            aggregator.contains(&format!("../mcp/{name}.yaml")),
            "services/config/config.yaml must include ../mcp/{name}.yaml"
        );
    }
    for name in ["jira", "confluence"] {
        assert!(
            !aggregator.contains(&format!("../mcp/{name}.yaml")),
            "services/config/config.yaml still includes the retired ../mcp/{name}.yaml"
        );
    }
}
