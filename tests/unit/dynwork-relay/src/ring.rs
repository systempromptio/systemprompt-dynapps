use std::time::{Duration, Instant};

use axum::body::Bytes;
use dynwork_relay::config::RingLimits;
use dynwork_relay::hub::Ring;

use crate::support::peer;

fn limits(envelopes: usize, bytes: usize) -> RingLimits {
    RingLimits {
        envelopes,
        bytes,
        ttl: Duration::from_secs(60),
    }
}

fn bytes(tag: u8, len: usize) -> Bytes {
    Bytes::from(vec![tag; len])
}

#[test]
fn the_oldest_envelope_is_evicted_past_the_count() {
    let now = Instant::now();
    let mut ring = Ring::new(limits(3, 1_000));
    for tag in 1..=4 {
        assert!(ring.push(peer(1), bytes(tag, 10), now));
    }
    assert_eq!(ring.len(), 3);
    assert_eq!(ring.evicted(), 1);
    let held: Vec<u8> = ring.take_for(peer(1), now).iter().map(|b| b[0]).collect();
    assert_eq!(held, vec![2, 3, 4]);
}

#[test]
fn envelopes_are_evicted_past_the_byte_bound() {
    let now = Instant::now();
    let mut ring = Ring::new(limits(100, 25));
    assert!(ring.push(peer(1), bytes(1, 10), now));
    assert!(ring.push(peer(1), bytes(2, 10), now));
    assert!(ring.push(peer(1), bytes(3, 10), now));
    assert_eq!(ring.len(), 2);
    assert_eq!(ring.bytes(), 20);
    assert!(!ring.push(peer(1), bytes(4, 26), now));
}

#[test]
fn envelopes_older_than_the_ttl_expire() {
    let now = Instant::now();
    let mut ring = Ring::new(limits(10, 1_000));
    assert!(ring.push(peer(1), bytes(1, 10), now));
    assert!(
        ring.take_for(peer(1), now + Duration::from_secs(61))
            .is_empty()
    );
    assert!(ring.is_empty());
    assert_eq!(ring.evicted(), 1);
}

#[test]
fn taking_returns_one_recipient_in_order_and_keeps_the_rest() {
    let now = Instant::now();
    let mut ring = Ring::new(limits(10, 1_000));
    for (to, tag) in [(1, 1), (2, 2), (1, 3), (2, 4)] {
        assert!(ring.push(peer(to), bytes(tag, 5), now));
    }
    let first: Vec<u8> = ring.take_for(peer(1), now).iter().map(|b| b[0]).collect();
    assert_eq!(first, vec![1, 3]);
    assert_eq!(ring.len(), 2);
    assert_eq!(ring.bytes(), 10);
}
