use std::time::Instant;

use axum::body::Bytes;
use dynwork_relay::config::RingLimits;
use dynwork_relay::envelope::{CONTROL_VERSION, PRESENCE_LEN, Presence, presence};
use dynwork_relay::hub::{CloseReason, DropReason, Endpoint, Hub, Routed};
use dynwork_relay::ids::{DeviceId, PeerKind};
use systemprompt::identifiers::UserId;

use crate::support::peer;

fn user(name: &str) -> UserId {
    UserId::try_new(name).expect("a test user id")
}

fn endpoint(owner: &str, byte: u8, kind: PeerKind) -> Endpoint {
    Endpoint {
        user: user(owner),
        peer: peer(byte),
        device: DeviceId::generate(),
        kind,
    }
}

fn hub() -> Hub {
    Hub::new(RingLimits::default(), 8)
}

fn payload(tag: u8) -> Bytes {
    Bytes::from(vec![tag; 40])
}

// The next queued message that is not a relay presence message.
fn next_envelope(rx: &mut tokio::sync::mpsc::Receiver<Bytes>) -> Option<Bytes> {
    loop {
        let message = rx.try_recv().ok()?;
        if message[0] != CONTROL_VERSION {
            return Some(message);
        }
    }
}

#[test]
fn a_phone_reaches_its_users_desktop() {
    let hub = hub();
    let now = Instant::now();
    let desktop = endpoint("alice", 1, PeerKind::Desktop);
    let phone = endpoint("alice", 2, PeerKind::Phone);
    let mut at_desktop = hub.attach(&desktop, now).expect("desktop attaches");
    let _at_phone = hub.attach(&phone, now).expect("phone attaches");
    assert_eq!(
        hub.route(&phone, desktop.peer, payload(7), now),
        Routed::Delivered
    );
    assert_eq!(next_envelope(&mut at_desktop.rx).expect("delivered")[0], 7);
    assert_eq!(hub.tunnels(), 2);
}

#[test]
fn another_users_desktop_is_unreachable_even_with_its_peer_id() {
    let hub = hub();
    let now = Instant::now();
    let desktop = endpoint("alice", 1, PeerKind::Desktop);
    let intruder = endpoint("mallory", 2, PeerKind::Phone);
    let mut at_desktop = hub.attach(&desktop, now).expect("desktop attaches");
    let _at_intruder = hub.attach(&intruder, now).expect("intruder attaches");
    assert_eq!(
        hub.route(&intruder, desktop.peer, payload(1), now),
        Routed::Dropped(DropReason::UnknownPeer)
    );
    assert!(at_desktop.rx.try_recv().is_err());
    assert_eq!(hub.held(&user("alice"), desktop.peer), 0);
}

#[test]
fn a_phone_cannot_reach_another_phone_nor_itself() {
    let hub = hub();
    let now = Instant::now();
    let first = endpoint("alice", 1, PeerKind::Phone);
    let second = endpoint("alice", 2, PeerKind::Phone);
    let _a = hub.attach(&first, now).expect("attaches");
    let _b = hub.attach(&second, now).expect("attaches");
    assert_eq!(
        hub.route(&first, second.peer, payload(1), now),
        Routed::Dropped(DropReason::SameKind)
    );
    assert_eq!(
        hub.route(&first, first.peer, payload(1), now),
        Routed::Dropped(DropReason::ToSelf)
    );
}

#[test]
fn envelopes_for_an_offline_peer_are_held_and_flushed_in_order() {
    let hub = hub();
    let now = Instant::now();
    let desktop = endpoint("alice", 1, PeerKind::Desktop);
    let phone = endpoint("alice", 2, PeerKind::Phone);
    let _at_desktop = hub.attach(&desktop, now).expect("desktop attaches");
    let mut at_phone = hub.attach(&phone, now).expect("phone attaches");
    hub.detach(&phone, at_phone.conn, &mut at_phone.rx);
    for tag in 1..=3 {
        assert_eq!(
            hub.route(&desktop, phone.peer, payload(tag), now),
            Routed::Held
        );
    }
    assert_eq!(hub.held(&desktop.user, desktop.peer), 3);
    let mut back = hub.attach(&phone, now).expect("phone attaches again");
    assert_eq!(back.flushed, 3);
    let order: Vec<u8> = (0..3)
        .map(|_| back.rx.try_recv().expect("held")[0])
        .collect();
    assert_eq!(order, vec![1, 2, 3]);
    assert_eq!(hub.held(&desktop.user, desktop.peer), 0);
}

#[test]
fn a_peer_never_seen_is_not_held_for() {
    let hub = hub();
    let now = Instant::now();
    let desktop = endpoint("alice", 1, PeerKind::Desktop);
    let _at = hub.attach(&desktop, now).expect("attaches");
    assert_eq!(
        hub.route(&desktop, peer(99), payload(1), now),
        Routed::Dropped(DropReason::UnknownPeer)
    );
}

#[test]
fn the_ring_evicts_the_oldest_held_envelope_when_full() {
    let hub = Hub::new(
        RingLimits {
            envelopes: 2,
            ..RingLimits::default()
        },
        8,
    );
    let now = Instant::now();
    let desktop = endpoint("alice", 1, PeerKind::Desktop);
    let phone = endpoint("alice", 2, PeerKind::Phone);
    let _d = hub.attach(&desktop, now).expect("attaches");
    let mut p = hub.attach(&phone, now).expect("attaches");
    hub.detach(&phone, p.conn, &mut p.rx);
    for tag in 1..=3 {
        assert_eq!(
            hub.route(&desktop, phone.peer, payload(tag), now),
            Routed::Held
        );
    }
    let mut back = hub.attach(&phone, now).expect("attaches");
    let order: Vec<u8> = (0..2)
        .map(|_| back.rx.try_recv().expect("held")[0])
        .collect();
    assert_eq!(order, vec![2, 3]);
}

#[tokio::test]
async fn revoking_a_device_closes_its_tunnel_and_refuses_it_again() {
    let hub = hub();
    let now = Instant::now();
    let phone = endpoint("alice", 2, PeerKind::Phone);
    let at_phone = hub.attach(&phone, now).expect("attaches");
    assert_eq!(hub.revoke(phone.device), 1);
    assert_eq!(at_phone.control.closed().await, CloseReason::Revoked);
    assert!(hub.attach(&phone, now).is_none());
    assert!(!hub.is_online(phone.device));
}

#[tokio::test]
async fn a_second_tunnel_for_the_same_peer_replaces_the_first() {
    let hub = hub();
    let now = Instant::now();
    let desktop = endpoint("alice", 1, PeerKind::Desktop);
    let mut first = hub.attach(&desktop, now).expect("attaches");
    let second = hub.attach(&desktop, now).expect("attaches");
    assert_eq!(first.control.closed().await, CloseReason::Replaced);
    hub.detach(&desktop, first.conn, &mut first.rx);
    assert!(hub.is_online(desktop.device));
    assert!(second.control.reason().is_none());
}

#[tokio::test]
async fn a_recipient_that_does_not_drain_its_queue_is_closed() {
    let hub = Hub::new(RingLimits::default(), 3);
    let now = Instant::now();
    let desktop = endpoint("alice", 1, PeerKind::Desktop);
    let phone = endpoint("alice", 2, PeerKind::Phone);
    let at_desktop = hub.attach(&desktop, now).expect("attaches");
    let _p = hub.attach(&phone, now).expect("attaches");
    assert_eq!(
        hub.route(&phone, desktop.peer, payload(1), now),
        Routed::Delivered
    );
    assert_eq!(
        hub.route(&phone, desktop.peer, payload(2), now),
        Routed::Delivered
    );
    assert_eq!(
        hub.route(&phone, desktop.peer, payload(3), now),
        Routed::Held
    );
    assert_eq!(at_desktop.control.closed().await, CloseReason::SlowConsumer);
}

#[test]
fn a_desktop_hears_when_its_users_phone_comes_and_goes() {
    let hub = hub();
    let now = Instant::now();
    let desktop = endpoint("alice", 1, PeerKind::Desktop);
    let phone = endpoint("alice", 2, PeerKind::Phone);
    let stranger = endpoint("mallory", 3, PeerKind::Phone);
    let mut at_desktop = hub.attach(&desktop, now).expect("attaches");
    let mut at_phone = hub.attach(&phone, now).expect("attaches");
    let _stranger = hub.attach(&stranger, now).expect("attaches");
    hub.detach(&phone, at_phone.conn, &mut at_phone.rx);
    let connected = at_desktop.rx.try_recv().expect("connected");
    assert_eq!(connected.len(), PRESENCE_LEN);
    assert_eq!(
        connected[..],
        presence(desktop.peer, phone.peer, Presence::Connected)[..]
    );
    let gone = at_desktop.rx.try_recv().expect("disconnected");
    assert_eq!(
        gone[..],
        presence(desktop.peer, phone.peer, Presence::Disconnected)[..]
    );
    assert!(
        at_desktop.rx.try_recv().is_err(),
        "nothing about another user's phone"
    );
}

#[test]
fn a_desktop_that_attaches_hears_which_phones_are_on() {
    let hub = hub();
    let now = Instant::now();
    let phone = endpoint("alice", 2, PeerKind::Phone);
    let desktop = endpoint("alice", 1, PeerKind::Desktop);
    let _at_phone = hub.attach(&phone, now).expect("attaches");
    let mut at_desktop = hub.attach(&desktop, now).expect("attaches");
    let message = at_desktop.rx.try_recv().expect("presence");
    assert_eq!(
        message[..],
        presence(desktop.peer, phone.peer, Presence::Connected)[..]
    );
}

#[test]
fn a_replaced_phone_socket_is_not_reported_as_gone() {
    let hub = hub();
    let now = Instant::now();
    let desktop = endpoint("alice", 1, PeerKind::Desktop);
    let phone = endpoint("alice", 2, PeerKind::Phone);
    let mut at_desktop = hub.attach(&desktop, now).expect("attaches");
    let mut first = hub.attach(&phone, now).expect("attaches");
    let _second = hub.attach(&phone, now).expect("attaches");
    hub.detach(&phone, first.conn, &mut first.rx);
    let events: Vec<u8> = std::iter::from_fn(|| at_desktop.rx.try_recv().ok())
        .map(|m| m[33])
        .collect();
    assert_eq!(
        events,
        vec![Presence::Connected as u8, Presence::Connected as u8]
    );
}

#[test]
fn what_a_closing_tunnel_never_wrote_is_held_for_its_reconnect_in_order() {
    let hub = hub();
    let now = Instant::now();
    let desktop = endpoint("alice", 1, PeerKind::Desktop);
    let phone = endpoint("alice", 2, PeerKind::Phone);
    let _d = hub.attach(&desktop, now).expect("attaches");
    let mut at_phone = hub.attach(&phone, now).expect("attaches");
    for tag in 1..=3 {
        let message = Bytes::from(dynwork_relay_envelope(phone.peer, desktop.peer, tag));
        assert_eq!(
            hub.route(&desktop, phone.peer, message, now),
            Routed::Delivered
        );
    }
    assert_eq!(hub.detach(&phone, at_phone.conn, &mut at_phone.rx), 3);
    let message = Bytes::from(dynwork_relay_envelope(phone.peer, desktop.peer, 4));
    assert_eq!(hub.route(&desktop, phone.peer, message, now), Routed::Held);
    let mut back = hub.attach(&phone, now).expect("attaches again");
    let tags: Vec<u8> = (0..4)
        .map(|_| next_envelope(&mut back.rx).expect("held")[37])
        .collect();
    assert_eq!(tags, vec![1, 2, 3, 4]);
}

#[test]
fn a_slow_consumers_queue_goes_back_ahead_of_what_was_held_after_it() {
    let hub = Hub::new(RingLimits::default(), 3);
    let now = Instant::now();
    let desktop = endpoint("alice", 1, PeerKind::Desktop);
    let phone = endpoint("alice", 2, PeerKind::Phone);
    let _d = hub.attach(&desktop, now).expect("attaches");
    let mut at_phone = hub.attach(&phone, now).expect("attaches");
    let mut routed = Vec::new();
    for tag in 1..=6 {
        let message = Bytes::from(dynwork_relay_envelope(phone.peer, desktop.peer, tag));
        routed.push(hub.route(&desktop, phone.peer, message, now));
    }
    assert_eq!(
        routed[3],
        Routed::Held,
        "the envelope that found the queue full is held"
    );
    hub.detach(&phone, at_phone.conn, &mut at_phone.rx);
    let mut back = hub.attach(&phone, now).expect("attaches again");
    let tags: Vec<u8> = std::iter::from_fn(|| next_envelope(&mut back.rx))
        .map(|m| m[37])
        .collect();
    assert_eq!(tags, vec![1, 2, 3, 4, 5, 6]);
}

#[test]
fn held_envelopes_from_several_desktops_all_reach_a_phone_with_a_small_queue() {
    let hub = Hub::new(
        RingLimits {
            envelopes: 4,
            ..RingLimits::default()
        },
        2,
    );
    let now = Instant::now();
    let phone = endpoint("alice", 9, PeerKind::Phone);
    let mut at_phone = hub.attach(&phone, now).expect("attaches");
    hub.detach(&phone, at_phone.conn, &mut at_phone.rx);
    for byte in [1, 2, 3] {
        let desktop = endpoint("alice", byte, PeerKind::Desktop);
        let _d = hub.attach(&desktop, now).expect("attaches");
        for tag in 1..=4 {
            assert_eq!(
                hub.route(&desktop, phone.peer, payload(tag), now),
                Routed::Held
            );
        }
    }
    let back = hub.attach(&phone, now).expect("attaches again");
    assert_eq!(back.flushed, 12);
}

#[test]
fn the_sweep_drops_rings_whose_envelopes_aged_out() {
    let hub = Hub::new(
        RingLimits {
            ttl: std::time::Duration::from_secs(1),
            ..RingLimits::default()
        },
        8,
    );
    let now = Instant::now();
    let desktop = endpoint("alice", 1, PeerKind::Desktop);
    let phone = endpoint("alice", 2, PeerKind::Phone);
    let _d = hub.attach(&desktop, now).expect("attaches");
    let mut at_phone = hub.attach(&phone, now).expect("attaches");
    hub.detach(&phone, at_phone.conn, &mut at_phone.rx);
    assert_eq!(
        hub.route(&desktop, phone.peer, payload(1), now),
        Routed::Held
    );
    assert_eq!(hub.rings(), 1);
    hub.sweep_rings(now);
    assert_eq!(hub.rings(), 1);
    hub.sweep_rings(now + std::time::Duration::from_secs(2));
    assert_eq!(hub.rings(), 0);
}

#[test]
fn revoking_an_offline_device_drops_its_ring_and_what_was_held_for_it() {
    let hub = hub();
    let now = Instant::now();
    let desktop = endpoint("alice", 1, PeerKind::Desktop);
    let phone = endpoint("alice", 2, PeerKind::Phone);
    let other = endpoint("alice", 3, PeerKind::Phone);
    let mut at_desktop = hub.attach(&desktop, now).expect("attaches");
    let mut at_phone = hub.attach(&phone, now).expect("attaches");
    let mut at_other = hub.attach(&other, now).expect("attaches");
    hub.detach(&phone, at_phone.conn, &mut at_phone.rx);
    hub.detach(&other, at_other.conn, &mut at_other.rx);
    assert_eq!(
        hub.route(&desktop, phone.peer, payload(1), now),
        Routed::Held
    );
    assert_eq!(
        hub.route(&desktop, other.peer, payload(2), now),
        Routed::Held
    );
    assert_eq!(hub.revoke(phone.device), 0);
    assert_eq!(
        hub.held(&desktop.user, desktop.peer),
        1,
        "only the other phone's envelope stays"
    );
    hub.detach(&desktop, at_desktop.conn, &mut at_desktop.rx);
    assert_eq!(hub.revoke(desktop.device), 0);
    assert_eq!(hub.held(&desktop.user, desktop.peer), 0);
    assert_eq!(hub.rings(), 0);
}

fn dynwork_relay_envelope(
    to: dynwork_relay::ids::PeerId,
    from: dynwork_relay::ids::PeerId,
    tag: u8,
) -> Vec<u8> {
    crate::support::envelope(to, from, &[tag; 3])
}
