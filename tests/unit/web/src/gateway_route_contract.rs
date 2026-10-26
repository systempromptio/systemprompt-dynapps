//! Where this deployment's gateway sends a Claude Code turn, resolved through
//! the catalog the committed `services/ai/*.yaml` resolve to.
//!
//! A route that matches Claude Code's client tools by name captures every
//! turn, because Claude Code declares its whole catalogue on every request.
//! The contract resolves real requests against the real files: every
//! `claude-*` request, whatever tools it declares, lands on `anthropic`,
//! `DeepSeek` answers only an explicit `deepseek-*` id, every `go-*` id lands
//! on the `OpenCode` Go provider for its wire format, and no route may match
//! on a client tool name.

use systemprompt::gateway::service::failover::{DeploymentState, plan_selection};
use systemprompt::identifiers::ModelId;
use systemprompt::manifest::services::{
    CatalogContext, ModelRequest, ResolvedCatalog, SelectionStrategy,
};
use systemprompt::wire::canonical::{CanonicalRequest, CanonicalTool};

use crate::support::committed_services;

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

fn catalog() -> ResolvedCatalog {
    committed_services()
        .resolve_catalog(&CatalogContext::default())
        .expect("the AI catalog resolves")
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
    request.tooling.tools = tools;
    request
}

fn endpoint_for(request: &CanonicalRequest) -> String {
    let catalog = catalog();
    let target = catalog
        .resolve(&ModelRequest::new(request.model.as_str()).with_request(request))
        .unwrap_or_else(|e| panic!("{} does not resolve: {e}", request.model));
    target
        .primary()
        .unwrap_or_else(|| panic!("no deployment for {}", request.model))
        .endpoint_ref()
        .to_string()
}

fn provider_for(request: &CanonicalRequest) -> String {
    endpoint_for(request)
        .split_once('/')
        .map(|(provider, _)| provider.to_owned())
        .unwrap_or_default()
}

#[test]
fn a_claude_code_turn_with_its_full_catalogue_stays_on_anthropic() {
    let catalogue = CLAUDE_CODE_CATALOGUE.map(client_tool).to_vec();
    assert_eq!(
        provider_for(&request("claude-sonnet-5-5", catalogue.clone())),
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
            provider_for(&request("claude-sonnet-5-5", vec![server_tool(kind, name)])),
            "anthropic",
            "{kind}"
        );
    }
}

#[test]
fn only_an_explicit_deepseek_id_reaches_deepseek() {
    assert_eq!(
        provider_for(&request("deepseek-flash", Vec::new())),
        "deepseek"
    );
    let catalogue = CLAUDE_CODE_CATALOGUE.map(client_tool).to_vec();
    assert_eq!(
        provider_for(&request("claude-sonnet-5-5", catalogue)),
        "anthropic",
        "a Claude turn never falls through to DeepSeek"
    );
}

#[test]
fn every_go_id_lands_on_the_opencode_go_endpoint_for_its_wire() {
    for (model, endpoint) in [
        ("go-minimax-m3", "opencode-go/messages"),
        ("go-qwen3.8-max", "opencode-go/messages"),
        ("go-qwen3.8-flash", "opencode-go/messages"),
        ("go-glm-5.3", "opencode-go/chat"),
        ("go-kimi-k3", "opencode-go/chat"),
        ("go-deepseek-v4-pro", "opencode-go/chat"),
        ("go-deepseek-v4.1-flash", "opencode-go/chat"),
        ("go-mimo-v2.6-pro", "opencode-go/chat"),
    ] {
        assert_eq!(
            endpoint_for(&request(model, Vec::new())),
            endpoint,
            "{model}"
        );
    }
    assert_eq!(
        provider_for(&request("deepseek-v4-pro", Vec::new())),
        "deepseek",
        "the bare deepseek-* id stays on the platform key"
    );
}

// Core attempts healthy deployments first, so a member's deployment on the
// platform id's chain would become the first attempt whenever the platform
// endpoint's breaker is open and bill the member's own plan.
#[test]
fn deepseek_v4_pro_never_lands_on_a_members_go_deployment_when_deepseek_is_down() {
    let catalog = catalog();
    let target = catalog
        .resolve(&ModelRequest::new("deepseek-v4-pro"))
        .expect("deepseek-v4-pro resolves");
    let hops: Vec<_> = target
        .chain
        .iter()
        .chain(&target.context_fallbacks)
        .collect();
    assert!(!hops.is_empty(), "deepseek-v4-pro has a deployment");
    let states: Vec<DeploymentState> = target
        .chain
        .iter()
        .map(|hop| DeploymentState {
            tripped: hop.provider.as_str() == "deepseek",
            weight: 1,
            in_flight: 0,
        })
        .collect();
    for strategy in [
        target.strategy,
        SelectionStrategy::Ordered,
        SelectionStrategy::Weighted,
        SelectionStrategy::LeastBusy,
    ] {
        for index in plan_selection(strategy, &states, 0) {
            let hop = &target.chain[index];
            assert!(
                !hop.credential.is_personal() && hop.provider.as_str() != "opencode-go",
                "{strategy:?} attempts {} for deepseek-v4-pro",
                hop.endpoint_ref()
            );
        }
    }
    for hop in hops {
        assert!(
            !hop.credential.is_personal(),
            "deepseek-v4-pro reaches {} on a member's key",
            hop.endpoint_ref()
        );
    }

    let go = catalog
        .resolve(&ModelRequest::new("go-deepseek-v4-pro"))
        .expect("go-deepseek-v4-pro resolves");
    assert!(
        go.chain
            .iter()
            .all(|hop| hop.credential.is_personal() && hop.provider.as_str() == "opencode-go"),
        "go-deepseek-v4-pro is served only on the member's Go key"
    );
    assert!(
        catalog
            .model("go-deepseek-v4-pro")
            .is_some_and(systemprompt::manifest::services::catalog::ResolvedModel::is_advertised),
        "go-deepseek-v4-pro is listed to clients"
    );
}

#[test]
fn no_route_matches_on_a_client_tool_name() {
    let catalog = catalog();
    let offenders: Vec<String> = catalog
        .routes()
        .iter()
        .filter_map(|route| route.when.as_ref().map(|when| (route.id.as_str(), when)))
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
