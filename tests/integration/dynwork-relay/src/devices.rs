use std::time::{Duration, Instant};

use axum::http::{Method, StatusCode};
use dynwork_relay::audit::AuditEvent;

use crate::support::{Relay, close_code, desktop_token, peer};

#[tokio::test]
async fn the_list_shows_the_users_devices_and_who_is_online() {
    let relay = Relay::start().await;
    let _desktop = relay.desktop("alice", peer(1)).await;
    let (phone_id, credential) = relay.pair_phone("alice", peer(2)).await;
    relay.pair_phone("mallory", peer(9)).await;
    let _phone = relay.phone(&credential).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    let (status, body) = relay
        .request(Method::GET, "/devices", Some(&desktop_token("alice")), None)
        .await;
    assert_eq!(status, StatusCode::OK);
    let devices = body.as_array().expect("a list");
    assert_eq!(devices.len(), 2);
    let phone = devices
        .iter()
        .find(|d| d["id"] == phone_id.as_str())
        .expect("the phone");
    assert_eq!(phone["kind"], "phone");
    assert_eq!(phone["name"], "Pixel");
    assert_eq!(phone["online"], true);
    assert!(phone.get("credential").is_none());
    let desktop = devices
        .iter()
        .find(|d| d["kind"] == "desktop")
        .expect("the desktop");
    assert_eq!(desktop["peer_id"], peer(1).to_hex());
}

#[tokio::test]
async fn another_users_device_cannot_be_revoked() {
    let relay = Relay::start().await;
    let (device, credential) = relay.pair_phone("alice", peer(2)).await;
    let path = format!("/devices/{device}");
    let (status, _) = relay
        .request(Method::DELETE, &path, Some(&desktop_token("mallory")), None)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = relay
        .request(
            Method::DELETE,
            "/devices/not-a-uuid",
            Some(&desktop_token("alice")),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    drop(relay.phone(&credential).await);
}

#[tokio::test]
async fn revoking_through_the_route_cuts_the_phone_within_five_seconds() {
    let relay = Relay::start().await;
    let (device, credential) = relay.pair_phone("alice", peer(2)).await;
    let mut phone = relay.phone(&credential).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    let started = Instant::now();
    let (status, _) = relay
        .request(
            Method::DELETE,
            &format!("/devices/{device}"),
            Some(&desktop_token("alice")),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(
        close_code(&mut phone, Duration::from_secs(5)).await,
        Some(4003)
    );
    assert!(started.elapsed() < Duration::from_secs(5));
    let (status, _) = relay
        .request(
            Method::DELETE,
            &format!("/devices/{device}"),
            Some(&desktop_token("alice")),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_revocation_made_elsewhere_is_cut_by_the_watch_within_five_seconds() {
    let relay = Relay::start().await;
    dynwork_relay::sweeper::spawn(&relay.state);
    let (device, credential) = relay.pair_phone("alice", peer(2)).await;
    let mut phone = relay.phone(&credential).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    let started = Instant::now();
    relay.store.revoke_directly(
        device.parse().expect("a device id"),
        relay.clock.now_plus(1),
    );
    assert_eq!(
        close_code(&mut phone, Duration::from_secs(5)).await,
        Some(4003)
    );
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[tokio::test]
async fn the_audit_trail_carries_ids_and_never_a_secret() {
    let relay = Relay::start().await;
    let token = relay.create_pairing("alice").await;
    let (_, body) = relay.claim(&token, peer(2)).await;
    let credential = body["credential"]
        .as_str()
        .expect("a credential")
        .to_owned();
    let device = body["device_id"].as_str().expect("an id").to_owned();
    let mut phone = relay.phone(&credential).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    relay
        .request(
            Method::DELETE,
            &format!("/devices/{device}"),
            Some(&desktop_token("alice")),
            None,
        )
        .await;
    assert_eq!(
        close_code(&mut phone, Duration::from_secs(5)).await,
        Some(4003)
    );
    drop(phone);
    let deadline = Instant::now() + Duration::from_secs(5);
    while !relay
        .audit
        .events()
        .iter()
        .any(|e| e.name() == "disconnect")
        && Instant::now() < deadline
    {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let events = relay.audit.events();
    let mut names: Vec<&str> = events.iter().map(AuditEvent::name).collect();
    assert_eq!(names[..3], ["pairing_created", "pair", "connect"]);
    names[3..].sort_unstable();
    assert_eq!(names[3..], ["disconnect", "revoke"]);
    let printed = format!("{events:?}");
    assert!(!printed.contains(&credential));
    assert!(!printed.contains(&token));
    assert!(printed.contains(&device));
    assert!(events.iter().any(|e| matches!(
        e,
        AuditEvent::Disconnect {
            reason: "revoked",
            ..
        }
    )));
}

#[tokio::test]
async fn a_desktop_is_not_revocable() {
    let relay = Relay::start().await;
    let _desktop = relay.desktop("alice", peer(1)).await;
    let (_, body) = relay
        .request(Method::GET, "/devices", Some(&desktop_token("alice")), None)
        .await;
    let id = body[0]["id"].as_str().expect("the desktop").to_owned();
    let (status, body) = relay
        .request(
            Method::DELETE,
            &format!("/devices/{id}"),
            Some(&desktop_token("alice")),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"], "desktop_not_revocable");
}
