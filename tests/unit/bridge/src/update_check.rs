//! A manual "Check for updates" asks the gateway to bypass its release cache;
//! the scheduled check keeps using it.

use systemprompt_identifiers::ValidatedUrl;
use systemprompt_bridge::gateway::GatewayClient;
use systemprompt_bridge::ids::BearerToken;
use wiremock::matchers::{method, path, query_param, query_param_is_missing};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> GatewayClient {
    crate::support::install_tls();
    GatewayClient::new(
        ValidatedUrl::try_new(server.uri()).expect("valid url"),
        reqwest::Client::new(),
    )
}

fn release(version: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "version": version,
        "sha256": "00",
        "size": 1,
        "notes_url": "https://example.invalid/notes",
    }))
}

async fn server() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/bridge/latest"))
        .and(query_param("refresh", "true"))
        .respond_with(release("9.9.9"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/bridge/latest"))
        .and(query_param_is_missing("refresh"))
        .respond_with(release("0.0.1"))
        .expect(1)
        .mount(&server)
        .await;
    server
}

#[tokio::test]
async fn a_manual_check_asks_the_gateway_to_skip_its_cache() {
    let server = server().await;
    let bearer = BearerToken::new("t");
    let fresh = client(&server)
        .fetch_latest_release_fresh(&bearer, "darwin-arm64")
        .await
        .expect("fresh release");
    assert_eq!(fresh.version, "9.9.9");

    let cached = client(&server)
        .fetch_latest_release(&bearer, "darwin-arm64")
        .await
        .expect("cached release");
    assert_eq!(cached.version, "0.0.1");
}

#[test]
fn brand_drift_within_a_minor_is_not_a_warning() {
    use systemprompt_bridge::brand::same_minor;
    assert!(same_minor("0.71.3", "0.71.0"));
    assert!(!same_minor("0.72.0", "0.71.0"));
    assert!(!same_minor("1.71.0", "0.71.0"));
    assert!(!same_minor("not-a-version", "0.71.0"));
}
