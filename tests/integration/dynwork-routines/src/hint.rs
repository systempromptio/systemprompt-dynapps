//! The change hint end to end: a routine write publishes a `NOTIFY`, the
//! relay's listener hands it to the hub, and the user's other desktops get
//! `{"t":"routines.changed"}` while the writing desktop does not.

use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::http::Method;
use dynapps_test_common::db_or_skip;
use dynwork_relay::config::RingLimits;
use dynwork_relay::envelope::{CONTROL_VERSION, PRESENCE_LEN};
use dynwork_relay::hint::listen;
use dynwork_relay::hub::{Endpoint, Hub};
use dynwork_relay::ids::{DeviceId, PeerId, PeerKind};
use serde_json::{Value, json};
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;

use crate::support::{Call, create_body, router, seed_user, user};

fn desktop(owner: &str, byte: u8) -> Endpoint {
    Endpoint {
        user: user(owner),
        peer: PeerId::from_bytes([byte; 16]),
        device: DeviceId::generate(),
        kind: PeerKind::Desktop,
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
async fn a_write_hints_the_users_other_desktops_but_not_the_writer() {
    let db = db_or_skip!();
    seed_user(&db.pool, "alice").await;
    let hub = Arc::new(Hub::new(RingLimits::default(), 8));
    let token = CancellationToken::new();
    let listener = tokio::spawn(listen(
        Arc::clone(&hub),
        Arc::clone(&db.pool),
        token.clone(),
    ));
    let writer = desktop("alice", 1);
    let mut writer_rx = hub.attach(&writer, Instant::now()).expect("attach").rx;
    let mut other_rx = hub
        .attach(&desktop("alice", 2), Instant::now())
        .expect("attach")
        .rx;
    let mut stranger_rx = hub
        .attach(&desktop("bob", 3), Instant::now())
        .expect("attach")
        .rx;
    let router = router(&db.pool);

    // The listener subscribes asynchronously: write until the hint arrives.
    let mut hint = None;
    for attempt in 0..20 {
        Call::new(Method::PUT, format!("/routines/r{attempt}"), "alice")
            .peer(&writer.peer.to_hex())
            .body(create_body("daily", "mac-1"))
            .send(&router)
            .await;
        hint = next_hint(&mut other_rx, Duration::from_millis(250)).await;
        if hint.is_some() {
            break;
        }
    }
    assert_eq!(hint, Some(json!({ "t": "routines.changed" })));
    assert_eq!(
        next_hint(&mut writer_rx, Duration::from_millis(300)).await,
        None
    );
    assert_eq!(
        next_hint(&mut stranger_rx, Duration::from_millis(50)).await,
        None
    );
    token.cancel();
    listener.await.expect("the listener stops");
}
