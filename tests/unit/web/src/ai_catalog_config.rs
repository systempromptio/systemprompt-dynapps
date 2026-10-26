//! The shipped AI catalog, pinned. `services/ai/models.yaml` and
//! `deployments.yaml` are the deployment's allow-list: a model id absent from
//! the resolved catalog cannot be called by anyone, and an id that is present
//! is billed at the pricing written beside it.
//!
//! Claude is served from Anthropic's own API under the bare `claude-*` ids,
//! listed to every Claude client. By customer requirement the only other
//! platform-key upstream is `DeepSeek`, kept for a cost comparison against
//! Claude and reachable only by an explicit `deepseek-*` id. `OpenCode` Go is
//! served only by each member's own key (a `personal` credential) under
//! `go-*` ids.
//!
//! That makes catalog drift expensive in two directions. A missing current
//! model is an outage for every client that names it. A stale price is a
//! silently wrong cost figure on every audit row, every budget window, and
//! every finance report derived from them — nothing downstream re-checks the
//! vendor's rate card.
//!
//! These tests resolve the committed `services/` tree exactly as the gateway
//! does and pin the served id set, the live-provider invariant, the
//! `DeepSeek`, Sonnet, Haiku and Opus 5.5 rate cards, and the fact that the
//! gateway's default model resolves to Anthropic alone.

use systemprompt::identifiers::SecretName;
use systemprompt::manifest::services::catalog::{ResolvedHop, ResolvedModel};
use systemprompt::manifest::services::{CatalogContext, ModelRequest, ResolvedCatalog};
use systemprompt::models::providers::ClientFamily;

use crate::support::committed_services;

const SERVED: [&str; 3] = ["claude-haiku-5-5", "claude-opus-5-5", "claude-sonnet-5-5"];
const LIVE_PROVIDERS: [&str; 4] = ["anthropic", "deepseek", "e2e-mock", "opencode-go"];

// Why: rate cards are YAML decimal literals parsed to the same `f64`.
fn same(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() < f64::EPSILON
}

fn catalog() -> ResolvedCatalog {
    committed_services()
        .resolve_catalog(&CatalogContext::default())
        .expect("the AI catalog resolves")
}

fn model<'a>(catalog: &'a ResolvedCatalog, id: &str) -> &'a ResolvedModel {
    catalog.model(id).unwrap_or_else(|| panic!("{id}"))
}

fn primary<'a>(catalog: &'a ResolvedCatalog, id: &str) -> &'a ResolvedHop {
    catalog
        .deployments(id)
        .first()
        .unwrap_or_else(|| panic!("{id} has no deployment"))
}

fn assert_rate_card(m: &ResolvedModel, input: f64, output: f64, cache_read: f64, cache_write: f64) {
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
    let catalog = catalog();
    let ids: Vec<&str> = catalog
        .models()
        .filter(|m| {
            catalog
                .deployments(m.id.as_str())
                .iter()
                .any(|hop| hop.provider.as_str() == "anthropic")
        })
        .map(|m| m.id.as_str())
        .collect();
    assert_eq!(ids, SERVED);
    for id in SERVED {
        assert_eq!(
            model(&catalog, id).upstream_model,
            id,
            "{id} is served by bare id"
        );
    }
}

// Why: Claude Desktop (via the bridge) only accepts `claude-*` ids listed to
// the Claude client family; an unlisted id leaves it with no model at all.
#[test]
fn the_anthropic_models_are_advertised_to_claude_clients() {
    let catalog = catalog();
    let listed: Vec<&str> = catalog
        .advertised_models(Some(ClientFamily::Claude))
        .map(|m| m.id.as_str())
        .collect();
    assert_eq!(listed, SERVED);
    let mock = model(&catalog, "e2e-mock-sonnet");
    assert!(!mock.is_advertised(), "the e2e mock lists to no client");
}

#[test]
fn the_retired_dated_ids_are_gone() {
    let catalog = catalog();
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
            catalog.resolve(&ModelRequest::new(retired)).is_err(),
            "{retired} is retired at the vendor, and the catalog still serves it: \
             serving it prices traffic against a rate card that no longer exists"
        );
    }
}

#[test]
fn sonnet_5_5_carries_the_current_rate_card() {
    assert_rate_card(model(&catalog(), "claude-sonnet-5-5"), 2.0, 10.0, 0.2, 2.5);
}

// Why: haiku-5-5 is tiered at 100K prompt tokens; the catalog carries the
// lower tier, and the >100K tier is $0.50 / $2.50.
#[test]
fn haiku_5_5_carries_its_base_tier_rate_card() {
    let catalog = catalog();
    let haiku = model(&catalog, "claude-haiku-5-5");
    assert_rate_card(haiku, 0.1, 0.5, 0.01, 0.125);
    assert_eq!(haiku.limits.context_window, 1_000_000);
    assert_eq!(haiku.limits.max_output_tokens, 128_000);
}

#[test]
fn opus_5_5_carries_its_launch_rate_card() {
    let catalog = catalog();
    let opus = model(&catalog, "claude-opus-5-5");
    assert_rate_card(opus, 4.0, 20.0, 0.2, 5.0);
    assert_eq!(opus.limits.context_window, 1_000_000);
    assert_eq!(opus.limits.max_output_tokens, 128_000);
}

// The default model is the bare catalog id, served by `anthropic` alone.
#[test]
fn the_gateway_default_model_is_served_by_anthropic_only() {
    let catalog = catalog();
    let default_model = catalog.default_model().expect("gateway.default_model");
    assert_eq!(default_model.as_str(), "claude-sonnet-5-5");

    let target = catalog
        .resolve(&ModelRequest::new(default_model.as_str()))
        .expect("the default model resolves");
    let providers: Vec<&str> = target.chain.iter().map(|h| h.provider.as_str()).collect();
    assert_eq!(providers, ["anthropic"], "no failover off Anthropic");
    assert_eq!(target.chain[0].upstream_model, "claude-sonnet-5-5");
}

#[test]
fn every_deployment_goes_to_a_live_provider() {
    let catalog = catalog();
    for m in catalog.models() {
        for hop in catalog.deployments(m.id.as_str()) {
            assert!(
                LIVE_PROVIDERS.contains(&hop.provider.as_str()),
                "{} reaches {}: only Anthropic, DeepSeek and OpenCode Go are live",
                m.id.as_str(),
                hop.provider.as_str()
            );
        }
    }
}

// Why: the customer requires their own Anthropic key; DeepSeek is the approved
// comparison.
#[test]
fn anthropic_and_deepseek_are_the_only_live_providers() {
    let catalog = catalog();
    let names: Vec<&str> = catalog.providers().map(|p| p.id.as_str()).collect();
    assert_eq!(names, LIVE_PROVIDERS);
}

#[test]
fn every_platform_hop_reads_a_secret_the_installation_holds() {
    let catalog = catalog();
    for m in catalog.models() {
        for hop in catalog
            .deployments(m.id.as_str())
            .iter()
            .filter(|hop| !hop.credential.is_personal())
        {
            let expected = if hop.provider.as_str() == "deepseek" {
                "deepseek"
            } else {
                "anthropic"
            };
            assert_eq!(
                hop.credential.platform_secret().map(SecretName::as_str),
                Some(expected),
                "{} on {} reads a secret this installation does not hold",
                m.id.as_str(),
                hop.provider.as_str()
            );
        }
    }
}

// Why: DeepSeek bills by time of day; the catalog carries the peak list rate
// and bills a cache write at the cache-miss rate (DeepSeek charges no write).
#[test]
fn deepseek_carries_its_peak_rate_card_and_only_deepseek_ids_reach_it() {
    let catalog = catalog();
    let deepseek = catalog.provider("deepseek").expect("deepseek provider");
    assert!(!deepseek.governance.european && !deepseek.governance.no_retain);
    assert_rate_card(model(&catalog, "deepseek-flash"), 0.3, 1.2, 0.006, 0.3);
    assert_rate_card(model(&catalog, "deepseek-v4-pro"), 1.32, 3.96, 0.044, 1.32);
    assert_eq!(
        primary(&catalog, "deepseek-flash").provider.as_str(),
        "deepseek"
    );
    for m in catalog.models() {
        let reaches_deepseek = catalog
            .deployments(m.id.as_str())
            .iter()
            .any(|hop| hop.provider.as_str() == "deepseek");
        assert_eq!(
            reaches_deepseek,
            m.id.as_str().starts_with("deepseek-"),
            "{} and DeepSeek",
            m.id.as_str()
        );
    }
}

#[test]
fn the_open_models_are_unrouted() {
    let catalog = catalog();
    for id in [
        "gpt-oss-120b",
        "gpt-4.1",
        "o4-mini",
        "gemini-3.8-flash",
        "vertex-gemini-3.8-flash",
        "zai.glm-5.2",
        "qwen.qwen3-coder-480b",
    ] {
        assert!(
            catalog.resolve(&ModelRequest::new(id)).is_err(),
            "{id} has no provider left to route to"
        );
    }
}

// Why: OpenCode Go is a member's personal subscription; a deployment secret
// serving it would bill one member's plan for everyone's requests.
#[test]
fn opencode_go_is_served_only_by_the_members_own_key() {
    let catalog = catalog();
    let go = catalog
        .provider("opencode-go")
        .expect("opencode-go provider");
    assert_eq!(go.forward_headers, ["x-opencode-session"]);
    let credential = catalog
        .credential(go.credential.as_str())
        .expect("opencode-go credential");
    assert!(
        credential.is_personal(),
        "opencode-go must be a personal credential"
    );
    assert!(credential.platform_secret().is_none());
    for m in catalog.models() {
        let first_on_go = primary(&catalog, m.id.as_str()).provider.as_str() == "opencode-go";
        if first_on_go {
            assert!(
                m.id.as_str().starts_with("go-"),
                "{} is served first by OpenCode Go under a non go-* id",
                m.id.as_str()
            );
        }
    }
}

// Why: Claude hosts list only the Claude family; DeepSeek and Go ids are
// listed to OpenCode (the openai family) only.
#[test]
fn deepseek_and_opencode_go_are_listed_to_opencode_but_not_to_claude_hosts() {
    let catalog = catalog();
    let openai: Vec<&str> = catalog
        .advertised_models(Some(ClientFamily::OpenAi))
        .map(|m| m.id.as_str())
        .collect();
    let claude: Vec<&str> = catalog
        .advertised_models(Some(ClientFamily::Claude))
        .map(|m| m.id.as_str())
        .collect();
    for id in [
        "deepseek-flash",
        "deepseek-v4-pro",
        "go-kimi-k3",
        "go-minimax-m3",
    ] {
        assert!(openai.contains(&id), "{id} must reach OpenCode");
        assert!(
            !claude.contains(&id),
            "{id} must stay out of the Claude hosts"
        );
    }
}
