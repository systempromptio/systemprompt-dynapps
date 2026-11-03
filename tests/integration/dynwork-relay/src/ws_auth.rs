use std::time::Duration;

use axum::http::Method;
use systemprompt::models::auth::{JwtAudience, Permission};

use crate::support::{
    Relay, Token, anonymous_token, desktop_token, foreign_token, peer, refused_status,
    token_with_audience,
};

#[tokio::test]
async fn a_desktop_socket_without_a_token_is_refused() {
    let relay = Relay::start().await;
    let request = tokio_tungstenite::connect_async(relay.ws_url("/desktop")).await;
    assert_eq!(refused_status(request.map(|(ws, _)| ws)), 401);
}

#[tokio::test]
async fn a_desktop_token_signed_by_another_deployment_is_refused() {
    let relay = Relay::start().await;
    let result = relay
        .connect_desktop(&foreign_token("alice"), Some(peer(1)))
        .await;
    assert_eq!(refused_status(result), 401);
}

#[tokio::test]
async fn a_token_of_another_audience_is_refused() {
    let relay = Relay::start().await;
    let mcp = token_with_audience("alice", JwtAudience::Mcp);
    assert_eq!(
        refused_status(relay.connect_desktop(&mcp, Some(peer(1))).await),
        401
    );
    let pairing = relay.create_pairing("alice").await;
    assert_eq!(
        refused_status(relay.connect_desktop(&pairing, Some(peer(1))).await),
        401
    );
}

#[tokio::test]
async fn a_desktop_must_name_its_peer_id() {
    let relay = Relay::start().await;
    assert_eq!(
        refused_status(relay.connect_desktop(&desktop_token("alice"), None).await),
        400
    );
}

#[tokio::test]
async fn a_desktop_of_an_account_that_no_longer_exists_is_refused() {
    let relay = Relay::start().await;
    let result = relay
        .connect_desktop(&desktop_token("ghost"), Some(peer(1)))
        .await;
    assert_eq!(refused_status(result), 401);
}

#[tokio::test]
async fn an_unknown_device_credential_is_refused() {
    let relay = Relay::start().await;
    let made_up = format!("dwr1_{}", "A".repeat(43));
    assert_eq!(refused_status(relay.connect_phone(&made_up).await), 401);
    assert_eq!(
        refused_status(relay.connect_phone("not-a-credential").await),
        401
    );
}

#[tokio::test]
async fn a_desktop_jwt_does_not_open_the_device_socket() {
    let relay = Relay::start().await;
    assert_eq!(
        refused_status(relay.connect_phone(&desktop_token("alice")).await),
        401
    );
}

#[tokio::test]
async fn a_revoked_credential_no_longer_opens_the_device_socket() {
    let relay = Relay::start().await;
    let (device, credential) = relay.pair_phone("alice", peer(2)).await;
    drop(relay.phone(&credential).await);
    let (status, _) = relay
        .request(
            Method::DELETE,
            &format!("/devices/{device}"),
            Some(&desktop_token("alice")),
            None,
        )
        .await;
    assert_eq!(status.as_u16(), 204);
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert_eq!(refused_status(relay.connect_phone(&credential).await), 401);
}

#[tokio::test]
async fn an_anonymous_site_session_is_not_a_desktop() {
    let relay = Relay::start().await;
    let anonymous = anonymous_token("alice");
    assert_eq!(
        refused_status(relay.connect_desktop(&anonymous, Some(peer(1))).await),
        401
    );
    let (status, _) = relay
        .request(Method::POST, "/pairings", Some(&anonymous), None)
        .await;
    assert_eq!(status.as_u16(), 401);
    let (status, _) = relay
        .request(Method::GET, "/devices", Some(&anonymous), None)
        .await;
    assert_eq!(status.as_u16(), 401);
}

#[tokio::test]
async fn service_plugin_and_purpose_bound_tokens_are_not_a_desktop() {
    let relay = Relay::start().await;
    let refused = [
        Token {
            permissions: vec![Permission::Service],
            ..Token::person("alice")
        }
        .mint(),
        Token {
            plugin: Some("dynapps-commons"),
            ..Token::person("alice")
        }
        .mint(),
        Token {
            purpose: Some("ci-export"),
            ..Token::person("alice")
        }
        .mint(),
    ];
    for token in refused {
        assert_eq!(
            refused_status(relay.connect_desktop(&token, Some(peer(1))).await),
            401
        );
        let (status, _) = relay
            .request(Method::POST, "/pairings", Some(&token), None)
            .await;
        assert_eq!(status.as_u16(), 401);
    }
}

#[tokio::test]
async fn a_user_and_an_admin_with_bridge_or_api_audience_are_desktops() {
    let relay = Relay::start().await;
    let admin = Token {
        permissions: vec![Permission::Admin],
        ..Token::person("alice")
    }
    .mint();
    let api = token_with_audience("alice", JwtAudience::Api);
    for (byte, token) in [(1, desktop_token("alice")), (2, admin), (3, api)] {
        drop(
            relay
                .connect_desktop(&token, Some(peer(byte)))
                .await
                .expect("a desktop connects"),
        );
    }
}

#[tokio::test]
async fn a_desktop_token_past_its_expiry_is_refused_even_within_the_leeway() {
    let relay = Relay::start().await;
    let token = desktop_token("alice");
    relay.clock.advance(chrono::Duration::minutes(31));
    assert_eq!(
        refused_status(relay.connect_desktop(&token, Some(peer(1))).await),
        401
    );
}
