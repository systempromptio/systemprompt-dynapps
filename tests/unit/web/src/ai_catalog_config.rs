//! The shipped Claude catalog, pinned. `services/ai/providers.yaml` is the
//! deployment's allow-list: `allow_unlisted_models` is `false` in
//! `services/ai/gateway.yaml`, so a model id that is absent from this file
//! cannot be called by anyone, and an id that is present is billed at the
//! pricing written beside it.
//!
//! Claude is served from Anthropic's own API under the bare `claude-*` ids,
//! listed to every client. Since 2026-10-16 the installation is Anthropic-only
//! by customer requirement: every other upstream (Gemini, Vertex, Cerebras) is
//! commented out of the file and its secret removed from the profiles. On
//! 2026-10-17 `DeepSeek` was added beside it (user-approved, for a cost
//! comparison against Claude), reachable only by an explicit `deepseek-*` id.
//!
//! That makes catalog drift expensive in two directions. A missing current
//! model is an outage for every client that names it. A stale price is a
//! silently wrong cost figure on every audit row, every budget window, and
//! every finance report derived from them — nothing downstream re-checks the
//! vendor's rate card.
//!
//! These tests pin the served id set, the live-provider invariant (Anthropic
//! and `DeepSeek`, one secret each, every route onto one of them), the
//! `DeepSeek` rate card, the Sonnet 5 rate that was
//! wrong here until 2026-09-04, the Opus 5.5 rate card, and the fact that the
//! gateway's default model resolves inside the catalog through the `claude-*`
//! route. Changing any of them is then a deliberate, review-visible edit rather
//! than a one-line drift.


use systemprompt::manifest::services::{
    GatewayConfigSpec, ProviderEntry, ProviderModel, ProviderRegistry,
};

use crate::support::repo_root;

const SERVED: [&str; 3] = ["claude-opus-5-5", "claude-sonnet-5-5", "claude-haiku-5-5"];

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
fn the_anthropic_lineup_is_exactly_the_served_claude_models() {
    let anthropic = provider("anthropic");
    let ids: Vec<String> = anthropic
        .models
        .iter()
        .map(|m| m.id.as_str().to_owned())
        .collect();
    assert_eq!(ids, SERVED);
    for id in SERVED {
        let served = anthropic
            .find_served_model(id)
            .unwrap_or_else(|| panic!("{id} finds no model on anthropic"));
        assert_eq!(served.upstream_model.as_deref(), Some(id));
    }
}

// Why: Claude Desktop (via the bridge) only accepts `claude-*` ids from an
// `anthropic`-surface provider; hidden ids leave it with no model at all.
#[test]
fn the_anthropic_models_are_advertised_to_clients() {
    let listed = registry().advertised_model_ids(&[]);
    for m in &provider("anthropic").models {
        assert!(!m.hidden, "{} must not be hidden", m.id.as_str());
        assert!(
            listed.iter().any(|id| id == m.id.as_str()),
            "{} must be advertised",
            m.id.as_str()
        );
    }
}

#[test]
fn the_retired_dated_ids_are_gone() {
    let models = provider("anthropic").models;
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
        "claude-haiku-4-5",
        "claude-haiku-4-5-20251001",
        "claude-opus-5",
        "claude-sonnet-5",
        "claude-fable-5",
        "claude-fable-5-1",
    ] {
        assert!(
            !models.iter().any(|m| m.matches(retired)),
            "{retired} is retired at the vendor, and anthropic still serves it: \
             serving it prices traffic against a rate card that no longer exists"
        );
    }
}

#[test]
fn sonnet_5_5_carries_the_current_rate_card() {
    let anthropic = provider("anthropic").models;
    assert_rate_card(model(&anthropic, "claude-sonnet-5-5"), 2.0, 10.0, 0.2, 2.5);
}

// Why: haiku-5-5 is tiered at 100K prompt tokens; the catalog carries the
// lower tier, and the >100K tier is $0.50 / $2.50.
#[test]
fn haiku_5_5_carries_its_base_tier_rate_card() {
    let anthropic = provider("anthropic").models;
    let haiku = model(&anthropic, "claude-haiku-5-5");
    assert_rate_card(haiku, 0.1, 0.5, 0.01, 0.125);
    assert_eq!(haiku.limits.context_window, 1_000_000);
    assert_eq!(haiku.limits.max_output_tokens, 128_000);
}

#[test]
fn opus_5_5_carries_its_launch_rate_card() {
    let anthropic = provider("anthropic").models;
    let opus = model(&anthropic, "claude-opus-5-5");
    assert_rate_card(opus, 4.0, 20.0, 0.2, 5.0);
    assert_eq!(opus.limits.context_window, 1_000_000);
    assert_eq!(opus.limits.max_output_tokens, 128_000);
}

// Why: since 2026-10-14 the gateway is Anthropic-only (a client's key, no
// Vertex): the default model routes to `anthropic` with no failover, and the
// provider serves the bare id directly.
#[test]
fn the_gateway_default_model_is_served_by_anthropic_only() {
    let spec = gateway();
    let default_model = spec.default_model.clone().expect("gateway.default_model");
    assert_eq!(default_model, "claude-sonnet-5-5[1m]");

    let route = spec
        .routes
        .iter()
        .find(|r| r.matches(&default_model) && r.when.is_none())
        .expect("a route for the default model");
    assert_eq!(route.provider.as_str(), "anthropic");
    assert!(route.fallbacks.is_empty(), "no failover off Anthropic");
    assert!(
        provider("anthropic")
            .find_served_model(&default_model)
            .is_some(),
        "anthropic must serve the default model by upstream name"
    );
}

#[test]
fn every_route_goes_to_a_live_provider() {
    for route in &gateway().routes {
        assert!(
            matches!(
                route.provider.as_str(),
                "anthropic" | "deepseek" | "e2e-mock"
            ),
            "route {:?} reaches {}: only Anthropic and DeepSeek are live",
            route.name,
            route.provider.as_str()
        );
    }
}

// Why: the customer requires that only their Anthropic API key is used; the
// other providers' secrets were deleted on 2026-10-16. DeepSeek was added on
// 2026-10-17 with the user's approval.
#[test]
fn anthropic_and_deepseek_are_the_only_live_providers() {
    let names: Vec<String> = registry()
        .providers
        .iter()
        .map(|p| p.name.as_str().to_owned())
        .collect();
    assert_eq!(names, ["anthropic", "deepseek", "e2e-mock"]);
}

#[test]
fn every_provider_reads_a_secret_the_installation_holds() {
    for p in &registry().providers {
        let expected = if p.name.as_str() == "deepseek" {
            "deepseek"
        } else {
            "anthropic"
        };
        assert_eq!(
            p.api_key_secret.as_str(),
            expected,
            "{} reads a secret this installation does not hold",
            p.name.as_str()
        );
    }
}

// Why: DeepSeek bills by time of day; the catalog carries the peak list rate
// and bills a cache write at the cache-miss rate (DeepSeek charges no write).
#[test]
fn deepseek_carries_its_peak_rate_card_and_only_deepseek_ids_reach_it() {
    let deepseek = provider("deepseek");
    assert!(!deepseek.governance.european && !deepseek.governance.no_retain);
    assert_rate_card(
        model(&deepseek.models, "deepseek-flash"),
        0.3,
        1.2,
        0.006,
        0.3,
    );
    assert_rate_card(
        model(&deepseek.models, "deepseek-v4-pro"),
        1.32,
        3.96,
        0.044,
        1.32,
    );
    let spec = gateway();
    let route = spec
        .routes
        .iter()
        .find(|r| r.matches("deepseek-flash"))
        .expect("a route for deepseek-flash");
    assert_eq!(route.provider.as_str(), "deepseek");
    assert!(
        !spec
            .routes
            .iter()
            .any(|r| r.matches("claude-sonnet-5-5") && r.provider.as_str() == "deepseek")
    );
}

#[test]
fn the_open_models_are_unrouted() {
    for pattern in [
        "gpt-oss-120b",
        "gpt-4.1",
        "o4-mini",
        "gemini-3.8-flash",
        "vertex-gemini-3.8-flash",
        "zai.glm-5.2",
        "qwen.qwen3-coder-480b",
    ] {
        assert!(
            !gateway().routes.iter().any(|r| r.matches(pattern)),
            "{pattern} has no provider left to route to"
        );
    }
}
