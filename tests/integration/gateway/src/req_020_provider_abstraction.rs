//! REQ-020 Model Provider Abstraction — "Applications use a provider-neutral
//! gateway/API so models/providers can be changed without application-layer
//! rewrites."
//!
//! Two providers speaking different wire protocols can serve the same catalog
//! model; which one serves it is decided by the order of its deployments in
//! `deployments.yaml`, or by a policy route that pins a deployment, and the
//! upstream name each endpoint receives is a per-deployment rewrite. Swapping
//! the provider is therefore a config edit the client never sees — the
//! register's low switching cost, demonstrated through the resolver the
//! gateway dispatches with.

use systemprompt::manifest::services::catalog::{ModelRequest, ResolvedCatalog};
use systemprompt::wire::WireProtocol;

use crate::support::{Fixture, governance};

fn two_providers() -> Fixture {
    Fixture::default()
        .provider("anthropic-upstream", "anthropic", governance(false, false))
        .provider("openai-compatible", "openai-chat", governance(false, false))
        .model("claude-large", None)
}

fn primary(catalog: &ResolvedCatalog, model: &str) -> (String, WireProtocol, String) {
    let target = catalog
        .resolve(&ModelRequest::new(model))
        .expect("the request resolves");
    let hop = target.primary().expect("a primary hop");
    (
        hop.provider.as_str().to_owned(),
        hop.wire,
        hop.upstream_model.clone(),
    )
}

#[test]
fn the_first_deployment_decides_the_provider() {
    let catalog = two_providers()
        .deploy(
            "claude-large",
            &["anthropic-upstream/api", "openai-compatible/api"],
        )
        .resolve()
        .expect("the catalog resolves");

    let (provider, wire, _) = primary(&catalog, "claude-large");
    assert_eq!(provider, "anthropic-upstream");
    assert!(matches!(wire, WireProtocol::Anthropic));
}

#[test]
fn reordering_deployments_substitutes_the_provider_without_touching_the_client_name() {
    let catalog = two_providers()
        .deploy(
            "claude-large",
            &["openai-compatible/api", "anthropic-upstream/api"],
        )
        .resolve()
        .expect("the catalog resolves");

    let (provider, wire, _) = primary(&catalog, "claude-large");
    assert_eq!(
        provider, "openai-compatible",
        "the first deployment wins, so the same external name now reaches the other provider"
    );
    assert!(
        matches!(wire, WireProtocol::OpenAiChat),
        "the substituted provider speaks a different wire protocol entirely"
    );
}

#[test]
fn a_policy_route_pins_the_provider_without_touching_the_catalog() {
    let catalog = two_providers()
        .deploy(
            "claude-large",
            &["anthropic-upstream/api", "openai-compatible/api"],
        )
        .route(
            "  - id: redirect\n    models: [\"claude-*\"]\n    deployments:\n    \
             - openai-compatible/api\n",
        )
        .resolve()
        .expect("the catalog resolves");

    let target = catalog
        .resolve(&ModelRequest::new("claude-large"))
        .expect("the request resolves");
    assert_eq!(
        target
            .route
            .as_ref()
            .map(systemprompt::identifiers::RouteId::as_str),
        Some("redirect")
    );
    assert_eq!(
        target.chain.len(),
        1,
        "the route narrows the chain to its pin"
    );
    assert_eq!(target.chain[0].provider.as_str(), "openai-compatible");
}

#[test]
fn upstream_model_rewrites_the_name_only_when_declared() {
    let catalog = two_providers()
        .deploy(
            "claude-large",
            &[
                "{endpoint: openai-compatible/api, upstream_model: oss-large}",
                "anthropic-upstream/api",
            ],
        )
        .resolve()
        .expect("the catalog resolves");

    let chain = catalog.deployments("claude-large");
    assert_eq!(chain[0].upstream_model, "oss-large");
    assert_eq!(
        chain[1].upstream_model, "claude-large",
        "without a rewrite the catalog id is forwarded as-is"
    );
}
