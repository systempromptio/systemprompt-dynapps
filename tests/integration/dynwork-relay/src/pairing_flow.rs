use axum::http::{Method, StatusCode};
use dynwork_relay::RelayLimits;
use serde_json::json;

use crate::support::{Relay, desktop_token, peer};

#[tokio::test]
async fn minting_a_pairing_needs_the_desktop_token() {
    let relay = Relay::start().await;
    let (status, body) = relay.request(Method::POST, "/pairings", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], "unauthorized");
}

#[tokio::test]
async fn a_claim_issues_a_credential_that_opens_the_device_socket() {
    let relay = Relay::start().await;
    let token = relay.create_pairing("alice").await;
    let (status, body) = relay.claim(&token, peer(2)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["peer_id"], peer(2).to_hex());
    let credential = body["credential"].as_str().expect("a credential");
    assert!(credential.starts_with("dwr1_"));
    drop(relay.phone(credential).await);
}

#[tokio::test]
async fn a_pairing_token_is_redeemed_once() {
    let relay = Relay::start().await;
    let token = relay.create_pairing("alice").await;
    assert_eq!(relay.claim(&token, peer(2)).await.0, StatusCode::CREATED);
    let (status, body) = relay.claim(&token, peer(3)).await;
    assert_eq!(status, StatusCode::GONE);
    assert_eq!(body["error"], "pairing_used");
}

#[tokio::test]
async fn an_expired_pairing_token_is_refused() {
    let relay = Relay::start().await;
    let token = relay.create_pairing("alice").await;
    relay.clock.advance(chrono::Duration::seconds(301));
    let (status, body) = relay.claim(&token, peer(2)).await;
    assert_eq!(status, StatusCode::GONE);
    assert_eq!(body["error"], "pairing_expired");
}

#[tokio::test]
async fn a_made_up_or_foreign_token_is_unknown() {
    let relay = Relay::start().await;
    let made_up = format!("dwp1_{}", "A".repeat(43));
    for token in [
        made_up,
        desktop_token("alice"),
        "eyJhbGciOiJSUzI1NiJ9.e30.c2ln".to_owned(),
    ] {
        let (status, body) = relay.claim(&token, peer(2)).await;
        assert_eq!(status, StatusCode::GONE);
        assert_eq!(body["error"], "pairing_unknown");
    }
}

#[tokio::test]
async fn the_pairing_token_is_opaque_and_never_in_a_uri() {
    let relay = Relay::start().await;
    let token = relay.create_pairing("alice").await;
    assert!(token.starts_with("dwp1_"));
    assert_eq!(token.len(), 5 + 43);
    assert!(!token.contains('.'), "not a JWT");
    assert!(!token.contains("alice"));
    let (status, _) = relay
        .request(
            Method::POST,
            &format!("/pairings/{token}/claim"),
            None,
            Some(json!({ "peer_id": peer(2).to_hex() })),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(relay.claim(&token, peer(2)).await.0, StatusCode::CREATED);
}

#[tokio::test]
async fn the_device_limit_refuses_a_claim_and_leaves_the_token_unused() {
    let relay = Relay::with_limits(RelayLimits {
        devices_per_user: 1,
        ..RelayLimits::default()
    })
    .await;
    relay.pair_phone("alice", peer(2)).await;
    let token = relay.create_pairing("alice").await;
    let (status, body) = relay.claim(&token, peer(3)).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"], "device_limit");
}

#[tokio::test]
async fn open_pairings_per_user_are_bounded() {
    let relay = Relay::with_limits(RelayLimits {
        open_pairings_per_user: 2,
        ..RelayLimits::default()
    })
    .await;
    relay.create_pairing("alice").await;
    relay.create_pairing("alice").await;
    let (status, body) = relay
        .request(
            Method::POST,
            "/pairings",
            Some(&desktop_token("alice")),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["error"], "too_many_pairings");
    relay.create_pairing("mallory").await;
}

#[tokio::test]
async fn a_claim_with_a_malformed_peer_id_or_name_is_refused() {
    let relay = Relay::start().await;
    let token = relay.create_pairing("alice").await;
    let (status, _) = relay
        .request(
            Method::POST,
            "/pairings/claim",
            None,
            Some(json!({ "token": token, "peer_id": "abc" })),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, _) = relay
        .request(
            Method::POST,
            "/pairings/claim",
            None,
            Some(json!({ "token": token, "peer_id": peer(2).to_hex(), "name": "x".repeat(65) })),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(relay.claim(&token, peer(2)).await.0, StatusCode::CREATED);
}

#[tokio::test]
async fn two_racing_claims_of_one_token_yield_one_device() {
    let relay = Relay::start().await;
    let token = relay.create_pairing("alice").await;
    let (a, b) = tokio::join!(relay.claim(&token, peer(2)), relay.claim(&token, peer(3)));
    let mut statuses = [a.0, b.0];
    statuses.sort_unstable();
    assert_eq!(statuses, [StatusCode::CREATED, StatusCode::GONE]);
}
