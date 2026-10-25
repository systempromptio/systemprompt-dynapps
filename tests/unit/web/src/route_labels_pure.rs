//! The route and provider labels every console page names a route by.
//!
//! A declared name wins; without one the label is derived from the route's
//! models and the display name of the provider its first deployment reaches,
//! and the route id never leaks into a label on its own.

use systemprompt::manifest::services::GatewayRoute;
use systemprompt_web_admin_core::repositories::gateway_catalog::{
    CATALOG_ROUTE_LABEL, derive_provider_label, derive_route_label, get_route_labels,
    resolved_catalog,
};

use crate::support::{committed_services, services_with_routes};

#[test]
fn provider_label_prefers_display_name_then_known_spelling_then_title_case() {
    let catalog = resolved_catalog(&committed_services()).expect("catalog resolves");
    let go = catalog.provider("opencode-go").expect("opencode-go");
    assert_eq!(
        derive_provider_label("opencode-go", Some(go)),
        "OpenCode Go"
    );

    let mut blank = go.clone();
    blank.display_name = Some("  ".to_owned());
    assert_eq!(derive_provider_label("vertex", Some(&blank)), "Vertex AI");
    assert_eq!(
        derive_provider_label("openai-responses", None),
        "OpenAI (Responses)"
    );
    assert_eq!(derive_provider_label("my-lab.host", None), "My Lab Host");
}

#[test]
fn route_label_reads_the_pattern_as_a_model_family() {
    assert_eq!(
        derive_route_label("claude-*", "Anthropic"),
        "Claude models · Anthropic"
    );
    assert_eq!(
        derive_route_label("*", "Anthropic"),
        "Everything else · Anthropic"
    );
    assert_eq!(derive_route_label("o4-mini", "OpenAI"), "O4 Mini · OpenAI");
    assert_eq!(
        derive_route_label("openai.gpt-oss-*", "Vertex AI MaaS"),
        "Openai Gpt Oss models · Vertex AI MaaS"
    );
}

#[test]
fn declared_name_wins_and_an_unnamed_route_is_derived() {
    let unnamed: GatewayRoute =
        serde_yaml::from_str("id: claude-family\nmodels: claude-*\n").expect("route parses");
    let labels = get_route_labels(&services_with_routes(vec![unnamed])).expect("labels resolve");

    let go = labels
        .find("go-deepseek-v4-pro")
        .expect("declared alias route");
    assert_eq!(go.label, "DeepSeek V4 Pro on OpenCode Go (member key)");
    assert!(go.declared_name);
    assert_eq!(go.provider, "opencode-go");
    assert_eq!(
        go.subtitle(),
        "go-deepseek-v4-pro → deepseek-v4-pro → OpenCode Go"
    );
    assert!(go.description.is_some());

    let claude = labels.find("claude-family").expect("unnamed route");
    assert_eq!(claude.label, "Claude models · Anthropic");
    assert!(!claude.declared_name);
    assert_eq!(claude.subtitle(), "claude-* → Anthropic");

    let catalog = labels.find("star-bf70e4").expect("the catalog route");
    assert_eq!(catalog.label, CATALOG_ROUTE_LABEL);

    assert_eq!(labels.label_of("no-such-route"), "no-such-route");
    assert_eq!(
        labels.find_provider("anthropic").map(|p| p.label.as_str()),
        Some("Anthropic")
    );
}
