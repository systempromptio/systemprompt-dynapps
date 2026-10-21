//! The outbound headers a `credential: user` provider receives.
//!
//! `OpenCode` Go admits a personal subscription only from a coding agent: it
//! wants its own `user-agent`, a stable `x-opencode-session`, and nothing of
//! Claude Code's `anthropic-beta` set. These pin that the provider's
//! `forward_headers` allowlist, `user_agent` and empty `accepted_betas` shape
//! the call exactly so, and that a platform provider never sees the header.

use systemprompt::ai::UpstreamTarget;
use systemprompt::identifiers::UserId;
use systemprompt::manifest::services::ProviderEntry;
use systemprompt::wire::WireProtocol;

const OPENCODE_UA: &str = "opencode/1.0 (systemprompt-gateway)";

fn provider(yaml: &str) -> ProviderEntry {
    serde_yaml::from_str(yaml).expect("provider entry parses")
}

fn personal_chat() -> ProviderEntry {
    provider(&format!(
        "name: opencode-go\nwire: openai_chat\nsurface: openai\n\
         endpoint: https://opencode.example.test/openai/v1\ncredential: user\n\
         user_agent: \"{OPENCODE_UA}\"\nforward_headers: [x-opencode-session]\n"
    ))
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
async fn personal_chat_provider_gets_session_user_agent_and_bearer_key() {
    let entry = personal_chat();
    let call = UpstreamTarget::resolve_personal(&entry, "go-key-a", &UserId::new("user-a"))
        .expect("personal target")
        .call()
        .await
        .expect("call");
    let headers = call.headers_forwarding(WireProtocol::OpenAiChat, &inbound());
    assert_eq!(value(&headers, "authorization"), Some("Bearer go-key-a"));
    assert_eq!(value(&headers, "x-opencode-session"), Some("ses_fixture"));
    assert_eq!(value(&headers, "user-agent"), Some(OPENCODE_UA));
    assert_eq!(value(&headers, "anthropic-beta"), None);
    assert_eq!(value(&headers, "anthropic-version"), None);
}

#[tokio::test]
async fn anthropic_wire_personal_provider_drops_every_beta() {
    let entry = provider(&format!(
        "name: opencode-go-anthropic\nwire: anthropic\nsurface: anthropic\n\
         endpoint: https://opencode.example.test/anthropic/v1\ncredential: user\n\
         user_agent: \"{OPENCODE_UA}\"\nforward_headers: [x-opencode-session]\n\
         accepted_betas: []\n"
    ));
    let call = UpstreamTarget::resolve_personal(&entry, "go-key-a", &UserId::new("user-a"))
        .expect("personal target")
        .call()
        .await
        .expect("call");
    let headers = call.headers_forwarding(WireProtocol::Anthropic, &inbound());
    assert_eq!(value(&headers, "x-api-key"), Some("go-key-a"));
    assert_eq!(value(&headers, "anthropic-version"), Some("2023-06-01"));
    assert_eq!(value(&headers, "anthropic-beta"), None);
    assert_eq!(value(&headers, "x-opencode-session"), Some("ses_fixture"));
    assert_eq!(value(&headers, "user-agent"), Some(OPENCODE_UA));
}

#[tokio::test]
async fn platform_provider_never_receives_the_session_header() {
    let entry = provider(
        "name: deepseek\nwire: openai_chat\nsurface: openai\n\
         endpoint: https://deepseek.example.test/v1\napi_key_secret: deepseek\n",
    );
    let call = UpstreamTarget::resolve(&entry, "platform-key")
        .expect("platform target")
        .call()
        .await
        .expect("call");
    let headers = call.headers_forwarding(WireProtocol::OpenAiChat, &inbound());
    assert_eq!(value(&headers, "x-opencode-session"), None);
    assert_eq!(value(&headers, "user-agent"), None);
}

#[test]
fn user_provider_is_never_served_from_a_deployment_secret() {
    let entry = personal_chat();
    assert!(entry.platform_secret().is_none());
    assert!(UpstreamTarget::from_secrets(&entry).is_err());
}

#[test]
fn platform_provider_without_a_secret_fails_validation() {
    let registry = systemprompt::manifest::services::ProviderRegistry {
        providers: vec![provider(
            "name: bare\nwire: openai_chat\nsurface: openai\nendpoint: https://bare.example.test/v1\n",
        )],
    };
    assert!(registry.validate().is_err());
}
