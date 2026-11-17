//! The change hint: the control frame's bytes, which desktops the hub hands
//! it to, and the `NOTIFY` payload the relay's listener delivers.

use std::time::Instant;

use dynwork_relay::config::RingLimits;
use dynwork_relay::envelope::{CONTROL_VERSION, PRESENCE_LEN, changed};
use dynwork_relay::hint::deliver;
use dynwork_relay::hub::{Attachment, Endpoint, Hub};
use dynwork_relay::ids::{DeviceId, PeerKind};
use serde_json::{Value, json};
use systemprompt::identifiers::UserId;

use crate::support::peer;

fn endpoint(owner: &str, byte: u8, kind: PeerKind) -> Endpoint {
    Endpoint {
        user: UserId::try_new(owner).expect("a test user id"),
        peer: peer(byte),
        device: DeviceId::generate(),
        kind,
    }
}

fn attach(hub: &Hub, endpoint: &Endpoint) -> Attachment {
    hub.attach(endpoint, Instant::now()).expect("attach")
}

// The hints queued for a socket, as their JSON bodies; presence is skipped.
fn hints(attachment: &mut Attachment) -> Vec<Value> {
    let mut found = Vec::new();
    while let Ok(message) = attachment.rx.try_recv() {
        if message[0] == CONTROL_VERSION && message.len() != PRESENCE_LEN {
            found.push(serde_json::from_slice(&message[17..]).expect("a JSON hint"));
        }
    }
    found
}

#[test]
fn the_frame_is_the_control_version_the_desktop_and_the_kind() {
    let frame = changed(peer(3), "routines");
    assert_eq!(frame[0], CONTROL_VERSION);
    assert_eq!(&frame[1..17], peer(3).as_bytes());
    assert_eq!(&frame[17..], br#"{"t":"routines.changed"}"#);
    assert_ne!(
        frame.len(),
        PRESENCE_LEN,
        "an old desktop must not read it as presence"
    );
}

#[test]
fn every_desktop_of_the_user_but_the_caller_gets_the_hint() {
    let hub = Hub::new(RingLimits::default(), 8);
    let caller = endpoint("alice", 1, PeerKind::Desktop);
    let other = endpoint("alice", 2, PeerKind::Desktop);
    let phone = endpoint("alice", 3, PeerKind::Phone);
    let stranger = endpoint("bob", 4, PeerKind::Desktop);
    let mut caller_rx = attach(&hub, &caller);
    let mut other_rx = attach(&hub, &other);
    let mut phone_rx = attach(&hub, &phone);
    let mut stranger_rx = attach(&hub, &stranger);

    let sent = hub.notify_user_desktops(&caller.user, Some(caller.peer), "routines");

    assert_eq!(sent, 1);
    assert_eq!(hints(&mut other_rx), vec![json!({"t": "routines.changed"})]);
    assert!(hints(&mut caller_rx).is_empty());
    assert!(hints(&mut phone_rx).is_empty());
    assert!(hints(&mut stranger_rx).is_empty());
}

#[test]
fn without_a_caller_peer_every_desktop_of_the_user_gets_it() {
    let hub = Hub::new(RingLimits::default(), 8);
    let one = endpoint("alice", 1, PeerKind::Desktop);
    let two = endpoint("alice", 2, PeerKind::Desktop);
    let mut one_rx = attach(&hub, &one);
    let mut two_rx = attach(&hub, &two);

    assert_eq!(hub.notify_user_desktops(&one.user, None, "chats"), 2);
    assert_eq!(hints(&mut one_rx), vec![json!({"t": "chats.changed"})]);
    assert_eq!(hints(&mut two_rx), vec![json!({"t": "chats.changed"})]);
}

#[test]
fn the_listener_delivers_a_notify_payload_and_drops_anything_else() {
    let hub = Hub::new(RingLimits::default(), 8);
    let caller = endpoint("alice", 1, PeerKind::Desktop);
    let other = endpoint("alice", 2, PeerKind::Desktop);
    let _caller_rx = attach(&hub, &caller);
    let mut other_rx = attach(&hub, &other);

    let payload = json!({
        "user": "alice",
        "except": caller.peer.to_hex(),
        "kind": "routines",
    })
    .to_string();
    assert_eq!(deliver(&hub, &payload), 1);
    assert_eq!(hints(&mut other_rx), vec![json!({"t": "routines.changed"})]);
    assert_eq!(deliver(&hub, "not a hint"), 0);
}
