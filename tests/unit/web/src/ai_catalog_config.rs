//! The shipped Claude catalog, pinned. `services/ai/providers.yaml` is the
//! deployment's allow-list: `allow_unlisted_models` is `false` in
//! `services/ai/gateway.yaml`, so a model id that is absent from this file
//! cannot be called by anyone, and an id that is present is billed at the
//! pricing written beside it.
//!
//! Claude is served twice. The bare `claude-*` ids belong to
//! `vertex-anthropic` (Claude on Vertex AI), which ordinary `claude-*` requests
//! use; native web and tool search route to Anthropic. Anthropic carries the
//! models as `anthropic-claude-*`, each naming the bare id as its upstream
//! model. That twin is what the route's failover lands on: core finds the
//! requested model on the fallback provider by upstream name, so a Vertex model
//! with no twin has nowhere to fail over to. The twins are `hidden`: they are
//! never listed to a client, so nobody picks an `anthropic-claude-*` id, while
//! the failover still finds them because it looks up by upstream name, not by
//! listing.
//!
//! That makes catalog drift expensive in two directions. A missing current
//! model is an outage for every client that names it. A stale price is a
//! silently wrong cost figure on every audit row, every budget window, and
//! every finance report derived from them — nothing downstream re-checks the
//! vendor's rate card.
//!
//! The Claude 4.x Opus and Sonnet models were removed on 2026-09-24, and
//! with them the direct `OpenAI` and Cerebras providers; every Vertex provider
//! now authenticates with the one `vertex` service-account secret.
//!
//! These tests pin the served id set and its twins, the Sonnet 5 rate that was
//! wrong here until 2026-09-04, the Opus 5.5 and Fable 5.1 rate cards on both
//! providers, and the fact that the gateway's default model resolves inside
//! the catalog through the `claude-*` route. Changing any of them is then a
//! deliberate, review-visible edit rather than a one-line drift.


use systemprompt::manifest::services::{
    GatewayConfigSpec, ProviderEntry, ProviderModel, ProviderRegistry,
};

use crate::support::repo_root;

const SERVED: [&str; 6] = [
    "claude-opus-5-5",
    "claude-opus-5",
    "claude-sonnet-5",
    "claude-fable-5-1",
    "claude-fable-5",
    "claude-haiku-4-5",
];

// Rate-card figures are written as decimal literals in the YAML and parsed to
// the same `f64`, so exact comparison is the right check here; the helper keeps
// clippy's blanket float-comparison lint from firing on every assertion.
fn same(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() < f64::EPSILON
}

fn registry() -> ProviderRegistry {
    let path = repo_root().join("services/ai/providers.yaml");
    let yaml = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
    let root: serde_yaml::Value =
        serde_yaml::from_str(&yaml).unwrap_or_else(|e| panic!("parsing {}: {e}", path.display()));
    let node = root
        .get("providers")
        .unwrap_or_else(|| panic!("{} has no providers: key", path.display()))
        .clone();
    serde_yaml::from_value(node)
        .unwrap_or_else(|e| panic!("parsing providers: in {}: {e}", path.display()))
}

fn gateway() -> GatewayConfigSpec {
    let path = repo_root().join("services/ai/gateway.yaml");
    let yaml = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
    let root: serde_yaml::Value =
        serde_yaml::from_str(&yaml).unwrap_or_else(|e| panic!("parsing {}: {e}", path.display()));
    let node = root
        .get("gateway")
        .unwrap_or_else(|| panic!("{} has no gateway: key", path.display()))
        .clone();
    serde_yaml::from_value(node)
        .unwrap_or_else(|e| panic!("parsing gateway: in {}: {e}", path.display()))
}

fn provider(name: &str) -> ProviderEntry {
    registry()
        .providers
        .into_iter()
        .find(|p| p.name.as_str() == name)
        .unwrap_or_else(|| panic!("the {name} provider entry"))
}

fn model<'a>(models: &'a [ProviderModel], id: &str) -> &'a ProviderModel {
    models
        .iter()
        .find(|m| m.id.as_str() == id)
        .unwrap_or_else(|| panic!("{id}"))
}

fn assert_rate_card(m: &ProviderModel, input: f64, output: f64, cache_read: f64, cache_write: f64) {
    let id = m.id.as_str();
    assert!(same(m.pricing.input_per_million, input), "{id} input");
    assert!(same(m.pricing.output_per_million, output), "{id} output");
    let read = m.pricing.cache_read_per_million.expect("cache read rate");
    let write = m.pricing.cache_write_per_million.expect("cache write rate");
    assert!(same(read, cache_read), "{id} cache read");
    assert!(same(write, cache_write), "{id} cache write");
}

#[test]
fn the_vertex_lineup_is_exactly_the_served_claude_models() {
    let ids: Vec<String> = provider("vertex-anthropic")
        .models
        .iter()
        .map(|m| m.id.as_str().to_owned())
        .collect();
    assert_eq!(ids, SERVED);
}

#[test]
fn every_served_claude_model_has_an_anthropic_api_twin_to_fail_over_to() {
    let anthropic = provider("anthropic");
    let twins: Vec<String> = anthropic
        .models
        .iter()
        .map(|m| m.id.as_str().to_owned())
        .collect();
    let mut expected: Vec<String> = SERVED.iter().map(|id| format!("anthropic-{id}")).collect();
    expected.push("anthropic-claude-haiku-4-5-20251001".to_owned());
    assert_eq!(twins, expected);
    let dated = model(&anthropic.models, "anthropic-claude-haiku-4-5-20251001");
    assert_eq!(
        dated.upstream_model.as_deref(),
        Some("claude-haiku-4-5-20251001")
    );
    assert!(dated.hidden);

    for id in SERVED {
        let twin = anthropic
            .find_served_model(id)
            .unwrap_or_else(|| panic!("failover from {id} finds no model on anthropic"));
        assert_eq!(twin.id.as_str(), format!("anthropic-{id}"));
        assert_eq!(twin.upstream_model.as_deref(), Some(id));
    }
}

#[test]
fn the_anthropic_api_twins_are_hidden_from_every_listing() {
    let registry = registry();
    let listed = registry.advertised_model_ids(&[]);
    for m in &provider("anthropic").models {
        assert!(m.hidden, "{} must be hidden", m.id.as_str());
        assert!(
            !listed.iter().any(|id| id == m.id.as_str()),
            "{} must not be advertised",
            m.id.as_str()
        );
    }
}

#[test]
fn the_retired_dated_ids_are_gone() {
    for name in ["vertex-anthropic", "anthropic"] {
        let models = provider(name).models;
        for retired in [
            "claude-opus-4-1-20250805",
            "claude-sonnet-4-20250514",
            "claude-opus-4-20250514",
            "claude-opus-4-5-20251101",
            "claude-sonnet-4-5-20250929",
            "claude-sonnet-4-5",
            "claude-opus-4-5",
            "claude-opus-4-6",
            "claude-sonnet-4-6",
            "claude-opus-4-7",
            "claude-opus-4-8",
        ] {
            assert!(
                !models.iter().any(|m| m.matches(retired)),
                "{retired} is retired at the vendor, and {name} still serves it: \
                 serving it prices traffic against a rate card that no longer exists"
            );
        }
    }
}

#[test]
fn the_dated_aliases_that_clients_still_pin_keep_resolving() {
    let models = provider("vertex-anthropic").models;
    // The catalog refresh left one dated id that shipped clients still pin.
    let alias = "claude-haiku-4-5-20251001";
    let haiku = models
        .iter()
        .find(|m| m.matches(alias))
        .unwrap_or_else(|| panic!("{alias} is pinned by shipped clients and must stay an alias"));
    assert_eq!(haiku.id.as_str(), "claude-haiku-4-5");
    assert_eq!(
        haiku.upstream_model.as_deref(),
        Some("claude-haiku-4-5@20251001")
    );
}

#[test]
fn sonnet_5_carries_the_current_rate_card() {
    let vertex = provider("vertex-anthropic").models;
    let anthropic = provider("anthropic").models;
    assert_rate_card(model(&vertex, "claude-sonnet-5"), 2.0, 10.0, 0.2, 2.5);
    assert_rate_card(
        model(&anthropic, "anthropic-claude-sonnet-5"),
        2.0,
        10.0,
        0.2,
        2.5,
    );
}

#[test]
fn opus_5_5_carries_its_launch_rate_card() {
    let vertex = provider("vertex-anthropic").models;
    let anthropic = provider("anthropic").models;
    for opus in [
        model(&vertex, "claude-opus-5-5"),
        model(&anthropic, "anthropic-claude-opus-5-5"),
    ] {
        assert_rate_card(opus, 4.0, 20.0, 0.2, 5.0);
        assert_eq!(opus.limits.context_window, 1_000_000);
        assert_eq!(opus.limits.max_output_tokens, 128_000);
    }
}

#[test]
fn fable_5_1_is_priced_and_sized_as_the_frontier_model() {
    let vertex = provider("vertex-anthropic").models;
    let anthropic = provider("anthropic").models;
    // Vertex publishes a lower cache-read rate for Fable 5.1 than Anthropic's
    // own API; each provider bills at its own card.
    assert_rate_card(model(&vertex, "claude-fable-5-1"), 10.0, 50.0, 0.25, 12.5);
    assert_rate_card(
        model(&anthropic, "anthropic-claude-fable-5-1"),
        10.0,
        50.0,
        1.0,
        12.5,
    );
    for fable in [
        model(&vertex, "claude-fable-5-1"),
        model(&anthropic, "anthropic-claude-fable-5-1"),
    ] {
        assert_eq!(fable.limits.context_window, 1_000_000);
        assert_eq!(fable.limits.max_output_tokens, 128_000);
    }
}

#[test]
fn the_gateway_default_model_is_a_catalog_entry() {
    let spec = gateway();
    let default_model = spec.default_model.clone().expect("gateway.default_model");
    assert_eq!(default_model, "claude-sonnet-5[1m]");

    let route = spec
        .routes
        .iter()
        .find(|r| r.matches(&default_model) && r.when.is_none())
        .expect("a route for the default model");
    assert_eq!(route.provider.as_str(), "vertex-anthropic");
    assert_eq!(
        route
            .fallback_provider
            .as_ref()
            .map(systemprompt::identifiers::ProviderId::as_str),
        Some("anthropic")
    );
    assert!(
        provider(route.provider.as_str())
            .models
            .iter()
            .any(|m| m.matches(&default_model)),
        "the default model must be listed: allow_unlisted_models is false, so \
         an unlisted default is a 403 for every request that omits a model"
    );
}

#[test]
fn the_removed_providers_stay_removed() {
    let registry = registry();
    for gone in ["openai", "openai-responses", "cerebras"] {
        assert!(
            !registry.providers.iter().any(|p| p.name.as_str() == gone),
            "{gone} was removed from the catalog on 2026-09-24"
        );
    }
    for pattern in ["gpt-oss-120b", "gpt-4.1", "o4-mini"] {
        assert!(
            !gateway().routes.iter().any(|r| r.matches(pattern)),
            "{pattern} has no provider left to route to"
        );
    }
}

#[test]
fn every_vertex_provider_shares_the_vertex_secret() {
    for name in [
        "vertex",
        "vertex-anthropic",
        "vertex-maas",
        "vertex-maas-us-central1",
    ] {
        assert_eq!(
            provider(name).api_key_secret.as_str(),
            "vertex",
            "{name}: one service-account JSON serves every Vertex provider"
        );
    }
}

#[test]
fn the_current_open_models_are_declared_and_routed_to_maas() {
    let spec = gateway();
    let maas = provider("vertex-maas").models;
    for id in [
        "zai.glm-5.2",
        "deepseek.v3.2",
        "moonshotai.kimi-k2-thinking",
        "qwen.qwen3-coder-480b",
    ] {
        assert!(maas.iter().any(|m| m.matches(id)), "{id} is not declared");
        let route = spec
            .routes
            .iter()
            .find(|r| r.matches(id))
            .unwrap_or_else(|| panic!("no route for {id}"));
        assert_eq!(route.provider.as_str(), "vertex-maas", "{id}");
    }
}
