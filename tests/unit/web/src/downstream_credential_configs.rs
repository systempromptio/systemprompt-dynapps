//! The committed downstream-credential configs are load-bearing and otherwise
//! unvalidated until boot: nothing else in the suite reads
//! `services/web/config/{atlassian,salesforce}.yaml` or the external MCP server
//! YAMLs that depend on them. These checks parse every one of them the way the
//! loaders do, so a typo'd key or an accessor path that disagrees with the
//! route it names fails here rather than at a user's first tool call.
//!
//! They also pin the shipped-disabled posture. Both integrations land ahead of
//! the credentials they need, and an accidental `enabled: true` would put a
//! server in front of users that can only fail.

use std::path::PathBuf;

use systemprompt_web_admin::{AtlassianConfig, SalesforceConfig};

use crate::support::repo_root;

// Why: must match the route registered by `atlassian_api_router` /
// `salesforce_api_router` under the `/api/public` nest. Core resolves the
// accessor as `api_external_url + token_endpoint`, so a disagreement here is a
// 404 at tool-call time and nowhere earlier.
const ATLASSIAN_ACCESSOR: &str = "/api/public/atlassian/token";
const SALESFORCE_ACCESSOR: &str = "/api/public/salesforce/token";

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
fn the_atlassian_config_parses_as_the_loader_reads_it() {
    let path = web_config("atlassian.yaml");
    let yaml = std::fs::read_to_string(&path).expect("atlassian.yaml is readable");
    let config: AtlassianConfig = serde_yaml::from_str(&yaml)
        .unwrap_or_else(|e| panic!("{} parses as AtlassianConfig: {e}", path.display()));

    // Why: `deny_unknown_fields` means parsing is the whole schema check, but it
    // cannot catch a config that parses and is still unusable.
    assert!(
        !config.redirect_uri.is_empty(),
        "atlassian.yaml must carry a redirect_uri; it has to match the app registration exactly"
    );
    assert!(
        config
            .redirect_uri
            .ends_with("/admin/auth/atlassian/callback"),
        "redirect_uri must point at the registered callback route, got {}",
        config.redirect_uri
    );
}

#[test]
fn the_atlassian_scopes_request_offline_access() {
    let yaml = std::fs::read_to_string(web_config("atlassian.yaml")).expect("readable");
    let config: AtlassianConfig = serde_yaml::from_str(&yaml).expect("parses");

    // Why: without `offline_access` Atlassian returns no refresh token at all,
    // and every link silently decays into a one-hour session. The default is
    // correct today; this pins it against a well-meaning edit.
    assert!(
        config
            .scopes
            .split_whitespace()
            .any(|s| s == "offline_access"),
        "atlassian scopes must include offline_access, got {:?}",
        config.scopes
    );
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
fn every_downstream_integration_ships_disabled() {
    // Why: each of these needs a credential that does not exist in any
    // environment yet. Enabling one before its secret is set puts a server in
    // front of users whose every tool call can only fail.
    for (file, name) in [
        ("jira.yaml", "jira"),
        ("confluence.yaml", "confluence"),
        ("salesforce.yaml", "salesforce"),
    ] {
        assert_eq!(
            server(file, name)
                .get("enabled")
                .and_then(serde_yaml::Value::as_bool),
            Some(false),
            "{file}: {name} must ship disabled until its credentials exist"
        );
    }

    for name in ["atlassian.yaml", "salesforce.yaml"] {
        let yaml = std::fs::read_to_string(web_config(name)).expect("readable");
        let value: serde_yaml::Value = serde_yaml::from_str(&yaml).expect("parses");
        assert_eq!(
            value.get("enabled").and_then(serde_yaml::Value::as_bool),
            Some(false),
            "{name} must ship disabled"
        );
    }
}

#[test]
fn every_external_server_points_at_the_accessor_that_serves_it() {
    for (file, name, accessor) in [
        ("jira.yaml", "jira", ATLASSIAN_ACCESSOR),
        ("confluence.yaml", "confluence", ATLASSIAN_ACCESSOR),
        ("salesforce.yaml", "salesforce", SALESFORCE_ACCESSOR),
    ] {
        let entry = server(file, name);
        let external_auth = entry
            .get("external_auth")
            .unwrap_or_else(|| panic!("{file}: {name} must declare external_auth"));

        assert_eq!(
            external_auth
                .get("token_endpoint")
                .and_then(serde_yaml::Value::as_str),
            Some(accessor),
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
fn the_atlassian_servers_use_the_streamable_http_endpoint() {
    // Why: core's MCP client speaks only StreamableHttp — it has no HTTP+SSE
    // transport — and Atlassian's /v1/sse form is deprecated after 30 June 2026.
    // A copied-from-the-docs /v1/sse endpoint would fail at connect time with
    // nothing pointing at the cause.
    for (file, name) in [("jira.yaml", "jira"), ("confluence.yaml", "confluence")] {
        let endpoint = server(file, name)
            .get("endpoint")
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_else(|| panic!("{file}: {name} must declare an endpoint"))
            .to_owned();
        assert!(
            !endpoint.ends_with("/sse"),
            "{file}: core has no SSE client transport; use the /v1/mcp endpoint, got {endpoint}"
        );
    }
}

#[test]
fn the_new_mcp_servers_are_listed_in_the_aggregator() {
    // Why: a flat resource file that is not in `includes:` is never loaded, and
    // nothing reports it — the server simply does not exist.
    let aggregator = std::fs::read_to_string(repo_root().join("services/config/config.yaml"))
        .expect("services/config/config.yaml is readable");
    for name in ["jira", "confluence", "salesforce"] {
        assert!(
            aggregator.contains(&format!("../mcp/{name}.yaml")),
            "services/config/config.yaml must include ../mcp/{name}.yaml"
        );
    }
}
