//! Where this deployment's gateway sends a Claude Code turn, resolved through
//! the shipped `services/ai/gateway.yaml` and `providers.yaml`.
//!
//! A route that matches Claude Code's client tools by name captures every
//! turn, because Claude Code declares its whole catalogue on every request.
//! The contract resolves real requests against the real files: every
//! `claude-*` request, whatever tools it declares, lands on `anthropic`,
//! `DeepSeek` answers only an explicit `deepseek-*` id, and no route may match
//! on a client tool name.

use systemprompt::identifiers::ModelId;
use systemprompt::loader::config_loader::gateway::backfill_route_ids;
use systemprompt::manifest::services::{GatewayConfig, GatewayConfigSpec, ProviderRegistry};
use systemprompt::wire::canonical::{CanonicalRequest, CanonicalTool};

use crate::support::repo_root;

const CLAUDE_CODE_CATALOGUE: [&str; 9] = [
    "Bash",
    "Read",
    "Edit",
    "Write",
    "Glob",
    "Grep",
    "WebSearch",
    "WebFetch",
    "ToolSearch",
];

fn yaml_section<T: serde::de::DeserializeOwned>(file: &str, key: &str) -> T {
    let path = repo_root().join(file);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
    let root: serde_yaml::Value =
        serde_yaml::from_str(&text).unwrap_or_else(|e| panic!("parsing {}: {e}", path.display()));
    let node = root
        .get(key)
        .unwrap_or_else(|| panic!("{} has no {key}: key", path.display()))
        .clone();
    serde_yaml::from_value(node)
        .unwrap_or_else(|e| panic!("parsing {key}: in {}: {e}", path.display()))
}

fn registry() -> ProviderRegistry {
    yaml_section("services/ai/providers.yaml", "providers")
}

fn gateway() -> GatewayConfig {
    let mut spec: GatewayConfigSpec = yaml_section("services/ai/gateway.yaml", "gateway");
    backfill_route_ids(&mut spec);
    spec.resolve()
}

fn client_tool(name: &str) -> CanonicalTool {
    CanonicalTool {
        name: name.to_owned(),
        anthropic_definition: Some(serde_json::json!({
            "name": name,
            "description": format!("{name} tool"),
            "input_schema": {"type": "object", "properties": {}},
        })),
        description: None,
        input_schema: serde_json::json!({"type": "object"}),
        cache_control: None,
    }
}

fn server_tool(kind: &str, name: &str) -> CanonicalTool {
    CanonicalTool {
        name: name.to_owned(),
        anthropic_definition: Some(serde_json::json!({"type": kind, "name": name})),
        description: None,
        input_schema: serde_json::Value::Null,
        cache_control: None,
    }
}

fn request(model: &str, tools: Vec<CanonicalTool>) -> CanonicalRequest {
    let mut request = CanonicalRequest::new(ModelId::new(model), Vec::new(), 1024);
    request.tools = tools;
    request
}

fn provider_for(request: &CanonicalRequest) -> String {
    gateway()
        .resolve_route(&registry(), request)
        .unwrap_or_else(|| panic!("no route for {}", request.model))
        .provider
        .to_string()
}

#[test]
fn a_claude_code_turn_with_its_full_catalogue_stays_on_anthropic() {
    let catalogue = CLAUDE_CODE_CATALOGUE.map(client_tool).to_vec();
    assert_eq!(
        provider_for(&request("claude-sonnet-5", catalogue.clone())),
        "anthropic"
    );
    assert_eq!(
        provider_for(&request("claude-opus-5-5", catalogue)),
        "anthropic"
    );
}

#[test]
fn every_server_tool_stays_on_anthropic() {
    for (kind, name) in [
        ("web_search_20250305", "web_search"),
        ("web_fetch_20260318", "web_fetch"),
        ("tool_search_tool_regex_20251119", "tool_search_tool_regex"),
        ("tool_search_tool_bm25_20251119", "tool_search_tool_bm25"),
    ] {
        assert_eq!(
            provider_for(&request("claude-sonnet-5", vec![server_tool(kind, name)])),
            "anthropic",
            "{kind}"
        );
    }
}

#[test]
fn only_an_explicit_deepseek_id_reaches_deepseek() {
    assert_eq!(
        provider_for(&request("deepseek-chat", Vec::new())),
        "deepseek"
    );
    let catalogue = CLAUDE_CODE_CATALOGUE.map(client_tool).to_vec();
    assert_eq!(
        provider_for(&request("claude-sonnet-5", catalogue)),
        "anthropic",
        "a Claude turn never falls through to DeepSeek"
    );
}

#[test]
fn no_route_matches_on_a_client_tool_name() {
    let offenders: Vec<String> = gateway()
        .routes
        .iter()
        .filter_map(|route| route.when.as_ref().map(|when| (route.effective_id(), when)))
        .flat_map(|(id, when)| {
            when.tool_names_any
                .iter()
                .filter(|name| name.chars().any(char::is_uppercase))
                .map(move |name| format!("{id}: {name}"))
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(
        offenders.is_empty(),
        "client tool names are CamelCase and are resent on every agent turn; \
         match server tools with tool_types_any instead: {offenders:?}"
    );
}
