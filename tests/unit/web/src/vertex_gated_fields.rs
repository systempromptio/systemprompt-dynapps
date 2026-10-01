//! What a Claude Code request body looks like when it leaves for Vertex.
//!
//! Claude Code sends `anthropic-beta: context-management-2025-06-27` with a
//! `context_management` body field. Vertex refuses the field without its flag
//! (`400 context_management: Extra inputs are not permitted`), so the gateway
//! either forwards both or neither: a flag the provider's policy drops takes
//! the field it gates with it. These tests run the request through the same
//! public steps the gateway's Anthropic adapter takes — the provider's beta
//! policy from `services/ai/providers.yaml`, `strip_fields_gated_by`, and the
//! Vertex body finish — and read what would reach the upstream.

use std::collections::BTreeSet;

use serde_json::{Map, Value, json};
use systemprompt::models::services::ProviderEntry;
use systemprompt::models::wire::anthropic::{
    AnthropicBeta, BetaHeader, BetaPolicy, strip_fields_gated_by,
};
use systemprompt::models::wire::upstream::UpstreamDialect;

use crate::vertex_accepted_betas::vertex_anthropic;

const CONTEXT_MANAGEMENT: &str = "context-management-2025-06-27";

// The `anthropic-beta` value Claude Code 2.1.283 sent (captured 2026-09-28).
const CLAUDE_CODE_BETAS: &str = "claude-code-20250219,interleaved-thinking-2025-05-14,\
thinking-token-count-2026-05-13,context-management-2025-06-27,\
prompt-caching-scope-2026-01-05,mid-conversation-system-2026-04-07,\
advisor-tool-2026-03-01,effort-2025-11-24";

fn claude_code_body() -> Map<String, Value> {
    let Value::Object(body) = json!({
        "model": "claude-haiku-4-5",
        "max_tokens": 1024,
        "messages": [{"role": "user", "content": "hi"}],
        "context_management": {"edits": [{"type": "clear_thinking_20251015"}]},
        "output_config": {"effort": "medium"},
    }) else {
        unreachable!("a JSON object literal")
    };
    body
}

fn policy_of(provider: &ProviderEntry) -> BetaPolicy {
    UpstreamDialect::of(provider.wire, &provider.endpoint)
        .beta_policy(provider.accepted_betas.as_ref())
}

fn forwarded(provider: &ProviderEntry) -> (BetaHeader, Map<String, Value>) {
    let policy = policy_of(provider);
    let sent = BetaHeader::parse(CLAUDE_CODE_BETAS);
    let dropped = sent.clone().refused_by(&policy);
    let mut body = claude_code_body();
    strip_fields_gated_by(&mut body, &dropped);
    UpstreamDialect::of(provider.wire, &provider.endpoint).finish_body(&mut body);
    (sent.admitted_by(&policy), body)
}

#[test]
fn vertex_anthropic_forwards_context_management_with_its_flag() {
    let (header, body) = forwarded(&vertex_anthropic());
    assert!(header.contains(CONTEXT_MANAGEMENT));
    assert!(body.contains_key("context_management"));
    assert!(!header.contains("advisor-tool-2026-03-01"));
}

#[test]
fn context_management_is_stripped_when_policy_drops_its_flag() {
    let mut provider = vertex_anthropic();
    let accepted: BTreeSet<AnthropicBeta> = provider
        .accepted_betas
        .take()
        .unwrap_or_default()
        .into_iter()
        .filter(|beta| !beta.as_str().starts_with("context-management-"))
        .collect();
    provider.accepted_betas = Some(accepted);

    let (header, body) = forwarded(&provider);

    assert!(!header.contains(CONTEXT_MANAGEMENT));
    assert!(!body.contains_key("context_management"));
    assert!(body.contains_key("output_config"));
    assert!(body.contains_key("messages"));
    assert_eq!(
        body.get("anthropic_version"),
        Some(&json!("vertex-2023-10-16"))
    );
    assert!(!body.contains_key("model"));
}

#[test]
fn a_vertex_provider_with_no_beta_list_sends_neither_flag_nor_field() {
    let mut provider = vertex_anthropic();
    provider.accepted_betas = None;

    let (header, body) = forwarded(&provider);

    assert!(header.is_empty());
    assert!(!body.contains_key("context_management"));
}
