//! The embedded Vertex rate card, cross-checked against this deployment's
//! gateway routes.
//!
//! Vertex publishes and retires `MaaS` models without an operator edit, so the
//! served catalog is not only what `services/ai/providers.yaml` names: at boot,
//! every model Vertex publishes that the core rate card prices is added to the
//! `vertex-maas` provider. A card id with no matching route in
//! `services/ai/gateway.yaml` is therefore priced but unreachable — the model
//! is installed into the registry and no request can ever select it, which is
//! invisible in the YAML because neither file mentions the other.
//!
//! This test makes that pairing explicit: the augmented registry must still
//! validate (so nothing the card prices can break the boot-time pricing gate),
//! and every card id that is not on its way out must match a route.


use systemprompt::loader::config_loader::gateway::backfill_route_ids;
use systemprompt::manifest::services::{GatewayConfigSpec, ProviderRegistry, VertexRateCard};

use crate::support::repo_root;

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

// Why: three routes in gateway.yaml carry no `id:`; the loader synthesises
// one at boot, so the test runs the same backfill before validating.
fn gateway() -> GatewayConfigSpec {
    let mut spec: GatewayConfigSpec = yaml_section("services/ai/gateway.yaml", "gateway");
    backfill_route_ids(&mut spec);
    spec
}

// The registry as it looks after a boot-time discovery pass in which Vertex
// publishes everything the rate card prices — the widest catalog this
// deployment can ever serve.
fn card() -> VertexRateCard {
    VertexRateCard::embedded().expect("the embedded rate card parses")
}

// Why: discovery appends a card entry to the provider the card names, so the
// registry is augmented the same way, provider by provider. Entries with a
// documented retirement are left out: discovery withholds them once inside
// the notice window, their catalog declarations are gone, and a route that
// reached only them would itself be refused at boot as reaching no priced
// model — so they are the one class of card id that may have no route.
fn augmented_registry() -> ProviderRegistry {
    let mut registry = registry();
    let card = card();
    for provider in &mut registry.providers {
        for entry in card.entries_for(provider.name.as_str()) {
            if entry.retires_on.is_some() || provider.models.iter().any(|m| m.id == entry.id) {
                continue;
            }
            provider.models.push(entry.to_provider_model());
        }
    }
    registry
}

#[test]
fn the_full_rate_card_still_validates_against_the_gateway() {
    let registry = augmented_registry();
    gateway()
        .resolve()
        .validate(&registry)
        .expect("every rate-card model must be priced and routable");
}

#[test]
fn native_tool_route_rejects_an_unpriced_upstream_model() {
    let mut registry = registry();
    let provider = registry
        .providers
        .iter_mut()
        .find(|provider| provider.name.as_str() == "anthropic")
        .expect("Anthropic provider is configured");
    let model = provider
        .models
        .iter_mut()
        .find(|model| model.upstream_model.as_deref() == Some("claude-sonnet-5-5"))
        .expect("Sonnet upstream model is configured");
    model.pricing = systemprompt::manifest::services::ai::ModelPricing::default();
    let error = gateway()
        .resolve()
        .validate(&registry)
        .expect_err("native routing must retain the billable-model check");
    assert!(error.to_string().contains("claude-native-tools"));
    assert!(error.to_string().contains("claude-sonnet-5-5"));
}

// Why: the Vertex rate card is still priced and discovered, but since
// 2026-10-14 the gateway is Anthropic-only, so no Vertex id may be reachable.
// Restoring Vertex is the moment to flip this back to "every id routes".
#[test]
fn no_rate_card_id_is_routable_while_anthropic_only() {
    let config = gateway().resolve();
    let routable: Vec<String> = card()
        .entries
        .iter()
        .filter(|e| e.retires_on.is_none())
        .map(|e| e.id.as_str().to_owned())
        .filter(|id| !id.starts_with("claude-") && config.find_route(id).is_some())
        .collect();
    assert!(
        routable.is_empty(),
        "Vertex rate-card models routable on an Anthropic-only gateway: {}",
        routable.join(", ")
    );
}
