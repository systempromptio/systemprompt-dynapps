//! The outbound headers a `personal` credential's endpoints receive.
//!
//! `OpenCode` Go admits a personal subscription only from a coding agent: it
//! wants its own `user-agent`, a stable `x-opencode-session`, and nothing of
//! Claude Code's `anthropic-beta` set. These pin, against the committed
//! catalog, that the provider's `forward_headers` allowlist, `user_agent` and
//! the `messages` endpoint's empty `accepted_betas` shape the call exactly so,
//! and that a platform provider never sees the header.

use std::sync::Arc;

use systemprompt::ai::{EndpointBinding, UpstreamTarget};
use systemprompt::identifiers::UserId;
use systemprompt::manifest::services::catalog::EndpointRef;
use systemprompt::manifest::services::{CatalogContext, ResolvedCatalog};
use systemprompt::security::credential::TokenCache;
use systemprompt::wire::WireProtocol;

use crate::support::committed_services;

const GO_UA: &str = "systemprompt-gateway (opencode-go; dynapps)";

fn catalog() -> ResolvedCatalog {
    committed_services()
        .resolve_catalog(&CatalogContext::default())
        .expect("the AI catalog resolves")
}

fn tokens() -> Arc<TokenCache> {
    Arc::new(TokenCache::default())
}

fn binding<'a>(catalog: &'a ResolvedCatalog, endpoint: &str) -> EndpointBinding<'a> {
    let endpoint = EndpointRef::parse(endpoint).expect("endpoint ref");
    EndpointBinding::of_endpoint(catalog, &endpoint).expect("endpoint binds")
}

fn inbound() -> Vec<(String, String)> {
    vec![
        ("x-opencode-session".to_owned(), "ses_fixture".to_owned()),
        (
            "anthropic-beta".to_owned(),
            "context-management-2025-06-27".to_owned(),
        ),
        ("anthropic-version".to_owned(), "2023-06-01".to_owned()),
    ]
}

fn value<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

#[tokio::test]
async fn personal_chat_endpoint_gets_session_user_agent_and_bearer_key() {
    let catalog = catalog();
    let call = UpstreamTarget::resolve_personal(
        &tokens(),
        binding(&catalog, "opencode-go/chat"),
        "go-key-a",
        &UserId::new("user-a"),
    )
    .expect("personal target")
    .call()
    .await
    .expect("call");
    let headers = call.headers_forwarding(WireProtocol::OpenAiChat, &inbound());
    assert_eq!(value(&headers, "authorization"), Some("Bearer go-key-a"));
    assert_eq!(value(&headers, "x-opencode-session"), Some("ses_fixture"));
    assert_eq!(value(&headers, "user-agent"), Some(GO_UA));
    assert_eq!(value(&headers, "anthropic-beta"), None);
    assert_eq!(value(&headers, "anthropic-version"), None);
}

#[tokio::test]
async fn anthropic_wire_personal_endpoint_drops_every_beta() {
    let catalog = catalog();
    let call = UpstreamTarget::resolve_personal(
        &tokens(),
        binding(&catalog, "opencode-go/messages"),
        "go-key-a",
        &UserId::new("user-a"),
    )
    .expect("personal target")
    .call()
    .await
    .expect("call");
    let headers = call.headers_forwarding(WireProtocol::Anthropic, &inbound());
    assert_eq!(value(&headers, "x-api-key"), Some("go-key-a"));
    assert_eq!(value(&headers, "anthropic-version"), Some("2023-06-01"));
    assert_eq!(value(&headers, "anthropic-beta"), None);
    assert_eq!(value(&headers, "x-opencode-session"), Some("ses_fixture"));
    assert_eq!(value(&headers, "user-agent"), Some(GO_UA));
}

#[tokio::test]
async fn platform_provider_never_receives_the_session_header() {
    let catalog = catalog();
    let hop = catalog
        .deployments("deepseek-flash")
        .first()
        .expect("deepseek-flash deployment");
    let call = UpstreamTarget::resolve(&tokens(), &catalog, hop, "platform-key")
        .expect("platform target")
        .call()
        .await
        .expect("call");
    let headers = call.headers_forwarding(WireProtocol::Anthropic, &inbound());
    assert_eq!(value(&headers, "x-opencode-session"), None);
    assert_eq!(value(&headers, "user-agent"), None);
}

#[test]
fn a_personal_endpoint_is_never_served_from_a_deployment_secret() {
    let catalog = catalog();
    let go = binding(&catalog, "opencode-go/chat");
    assert!(go.credential.platform_secret().is_none());
    assert!(UpstreamTarget::from_secrets(&tokens(), &no_platform_secrets(), go).is_err());
}

#[test]
fn a_platform_credential_without_a_secret_fails_to_resolve() {
    let mut services = committed_services();
    services
        .credentials
        .credentials
        .values_mut()
        .filter(|entry| !entry.kind.is_personal())
        .for_each(|entry| entry.secret = None);
    assert!(
        services
            .resolve_catalog(&CatalogContext::default())
            .is_err()
    );
}

fn no_platform_secrets() -> systemprompt::manifest::secrets::Secrets {
    serde_json::from_value(serde_json::json!({
        "database_url": "postgres://unused:unused@localhost:5432/postgres",
        "oauth_at_rest_pepper": "unit-test-pepper-not-a-real-secret",
    }))
    .expect("a secrets document with no platform keys")
}
