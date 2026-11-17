//! The templates change hint end to end: a publish sends a `NOTIFY` the relay
//! hands to every connected desktop, of every person, as
//! `{"t":"templates.changed"}`. Phones never get it.

use std::sync::Arc;
use std::time::{Duration, Instant};

use dynapps_test_common::db_or_skip;
use dynwork_relay::config::RingLimits;
use dynwork_relay::envelope::{CONTROL_VERSION, PRESENCE_LEN};
use dynwork_relay::hint::listen;
use dynwork_relay::hub::{Endpoint, Hub};
use dynwork_relay::ids::{DeviceId, PeerId, PeerKind};
use serde_json::{Value, json};
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;

use crate::support::user;
use crate::template_support::{publish, seed_admin};

fn endpoint(owner: &str, byte: u8, kind: PeerKind) -> Endpoint {
    Endpoint {
        user: user(owner),
        peer: PeerId::from_bytes([byte; 16]),
        device: DeviceId::generate(),
        kind,
    }
}

async fn next_hint(rx: &mut Receiver<axum::body::Bytes>, wait: Duration) -> Option<Value> {
    let deadline = tokio::time::Instant::now() + wait;
    loop {
        let message = tokio::time::timeout_at(deadline, rx.recv()).await.ok()??;
        if message[0] == CONTROL_VERSION && message.len() != PRESENCE_LEN {
            return Some(serde_json::from_slice(&message[17..]).expect("a JSON hint"));
        }
    }
}

#[tokio::test]
async fn a_publish_hints_every_connected_desktop() {
    let db = db_or_skip!();
    let admin = seed_admin(&db.pool, "admin").await;
    let hub = Arc::new(Hub::new(RingLimits::default(), 8));
    let token = CancellationToken::new();
    let listener = tokio::spawn(listen(
        Arc::clone(&hub),
        Arc::clone(&db.pool),
        token.clone(),
    ));
    let mut alice_rx = hub
        .attach(&endpoint("alice", 1, PeerKind::Desktop), Instant::now())
        .expect("attach")
        .rx;
    let mut bob_rx = hub
        .attach(&endpoint("bob", 2, PeerKind::Desktop), Instant::now())
        .expect("attach")
        .rx;
    let mut phone_rx = hub
        .attach(&endpoint("bob", 3, PeerKind::Phone), Instant::now())
        .expect("attach")
        .rx;

    // The listener subscribes asynchronously: publish until the hint arrives.
    let mut hint = None;
    for attempt in 0..20 {
        publish(
            &db.pool,
            &admin,
            "daily",
            &["sales"],
            &format!("Daily {attempt}"),
        )
        .await;
        hint = next_hint(&mut alice_rx, Duration::from_millis(250)).await;
        if hint.is_some() {
            break;
        }
    }
    assert_eq!(hint, Some(json!({ "t": "templates.changed" })));
    assert_eq!(
        next_hint(&mut bob_rx, Duration::from_millis(300)).await,
        Some(json!({ "t": "templates.changed" })),
        "another person's desktop gets it too"
    );
    assert_eq!(
        next_hint(&mut phone_rx, Duration::from_millis(50)).await,
        None
    );
    token.cancel();
    listener.await.expect("the listener stops");
}
