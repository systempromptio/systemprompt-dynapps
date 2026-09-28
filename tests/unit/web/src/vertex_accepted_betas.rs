//! The `anthropic-beta` flags `vertex-anthropic` forwards, pinned.
//!
//! The gateway sends a Vertex provider exactly the flags in its
//! `accepted_betas`; a flag that gates a body field must be among them or the
//! field reaches Vertex alone and every request is refused. On 2026-09-28 the
//! list was empty and Claude Code 2.1.283 got `400 context_management: Extra
//! inputs are not permitted` on every prompt. Removing a flag from the list is
//! therefore a review-visible act, and the flags a served Claude Code sends
//! that Vertex serves (Anthropic's feature matrix, read 2026-09-28) are named
//! here one by one.

use systemprompt::models::services::{ProviderEntry, ProviderRegistry};
use systemprompt::models::wire::anthropic::AnthropicBeta;

use crate::support::repo_root;

fn vertex_anthropic() -> ProviderEntry {
    let path = repo_root().join("services/ai/providers.yaml");
    let yaml = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
    let root: serde_yaml::Value =
        serde_yaml::from_str(&yaml).unwrap_or_else(|e| panic!("parsing {}: {e}", path.display()));
    let registry: ProviderRegistry = serde_yaml::from_value(
        root.get("providers")
            .unwrap_or_else(|| panic!("{} has no providers: key", path.display()))
            .clone(),
    )
    .unwrap_or_else(|e| panic!("parsing providers: in {}: {e}", path.display()));
    registry
        .providers
        .into_iter()
        .find(|p| p.name.as_str() == "vertex-anthropic")
        .expect("the vertex-anthropic provider entry")
}

#[test]
fn vertex_anthropic_forwards_every_beta_claude_code_needs_served() {
    let declared = vertex_anthropic()
        .accepted_betas
        .expect("vertex-anthropic declares accepted_betas");
    for flag in [
        "context-management-2025-06-27",
        "interleaved-thinking-2025-05-14",
        "effort-2025-11-24",
    ] {
        assert!(
            declared.contains(&AnthropicBeta::new(flag)),
            "{flag} must be forwarded to Vertex"
        );
    }
}

// Why a prefix and not the one flag above: the gateway strips a body field
// whose gating beta it drops, so the moment Claude Code moves to a newer
// `context-management-*` date this list must move with it or every request
// loses `context_management` instead of being served it.
#[test]
fn a_context_management_beta_is_declared_so_the_body_field_is_served_not_stripped() {
    let declared = vertex_anthropic()
        .accepted_betas
        .expect("vertex-anthropic declares accepted_betas");
    assert!(
        declared
            .iter()
            .any(|b| b.as_str().starts_with("context-management-")),
        "a context-management-* flag must be declared for vertex-anthropic"
    );
}

#[test]
fn vertex_anthropic_declares_nothing_vertex_does_not_serve() {
    let declared = vertex_anthropic()
        .accepted_betas
        .expect("vertex-anthropic declares accepted_betas");
    for prefix in [
        "advisor-tool-",
        "code-execution-",
        "mcp-client-",
        "files-api-",
        "skills-",
        "web-fetch-",
        "context-1m-",
    ] {
        assert!(
            !declared.iter().any(|b| b.as_str().starts_with(prefix)),
            "{prefix}* is not served on Vertex (feature matrix, 2026-09-28)"
        );
    }
}
