use std::time::Duration;

use dynwork_relay::RelayLimits;
use dynwork_relay::config::RateLimit;
use dynwork_relay::envelope::{MAX_CIPHERTEXT, MAX_ENVELOPE};
use futures_util::SinkExt;
use tokio_tungstenite::tungstenite::Message;

use crate::support::{
    Relay, close_code, desktop_token, envelope, peer, recv, refused_status, send,
};

const WAIT: Duration = Duration::from_secs(5);

#[tokio::test]
async fn the_largest_envelope_passes_and_a_larger_one_closes_the_socket() {
    let relay = Relay::start().await;
    let mut desktop = relay.desktop("alice", peer(1)).await;
    let (_, credential) = relay.pair_phone("alice", peer(2)).await;
    let mut phone = relay.phone(&credential).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    let largest = envelope(peer(1), peer(2), 0, &vec![7; MAX_CIPHERTEXT]);
    assert_eq!(largest.len(), MAX_ENVELOPE);
    send(&mut phone, largest).await;
    assert_eq!(
        recv(&mut desktop, WAIT).await.map(|e| e.len()),
        Some(MAX_ENVELOPE)
    );
    send(
        &mut phone,
        envelope(peer(1), peer(2), 1, &vec![7; MAX_CIPHERTEXT + 1]),
    )
    .await;
    assert_eq!(close_code(&mut phone, WAIT).await, Some(1009));
}

#[tokio::test]
async fn an_envelope_with_no_ciphertext_closes_the_socket() {
    let relay = Relay::start().await;
    let mut desktop = relay.desktop("alice", peer(1)).await;
    send(&mut desktop, envelope(peer(2), peer(1), 0, &[])).await;
    assert_eq!(close_code(&mut desktop, WAIT).await, Some(1008));
}

#[tokio::test]
async fn a_sender_over_its_rate_is_closed() {
    let relay = Relay::with_limits(RelayLimits {
        rate: RateLimit {
            messages_per_second: 1.0,
            message_burst: 5.0,
            ..RateLimit::default()
        },
        ..RelayLimits::default()
    })
    .await;
    let mut desktop = relay.desktop("alice", peer(1)).await;
    for seq in 0..10u32 {
        send(&mut desktop, envelope(peer(2), peer(1), seq, b"flood")).await;
    }
    assert_eq!(close_code(&mut desktop, WAIT).await, Some(1008));
}

#[tokio::test]
async fn a_text_frame_closes_the_socket() {
    let relay = Relay::start().await;
    let mut desktop = relay.desktop("alice", peer(1)).await;
    desktop
        .send(Message::text("hello"))
        .await
        .expect("send text");
    assert_eq!(close_code(&mut desktop, WAIT).await, Some(1003));
}

#[tokio::test]
async fn a_silent_peer_is_closed_by_the_heartbeat() {
    let relay = Relay::with_limits(RelayLimits {
        heartbeat_interval: Duration::from_millis(100),
        heartbeat_timeout: Duration::from_millis(350),
        ..RelayLimits::default()
    })
    .await;
    let mut desktop = relay.desktop("alice", peer(1)).await;
    tokio::time::sleep(Duration::from_millis(800)).await;
    assert_eq!(close_code(&mut desktop, WAIT).await, Some(4008));
}

#[tokio::test]
async fn a_peer_that_answers_pings_stays_connected() {
    let relay = Relay::with_limits(RelayLimits {
        heartbeat_interval: Duration::from_millis(100),
        heartbeat_timeout: Duration::from_millis(350),
        ..RelayLimits::default()
    })
    .await;
    let mut desktop = relay.desktop("alice", peer(1)).await;
    assert_eq!(
        close_code(&mut desktop, Duration::from_millis(900)).await,
        None
    );
}

#[tokio::test]
async fn the_session_lifetime_closes_with_a_reconnect_hint() {
    let relay = Relay::with_limits(RelayLimits {
        session_lifetime: Duration::from_millis(300),
        ..RelayLimits::default()
    })
    .await;
    let mut desktop = relay.desktop("alice", peer(1)).await;
    assert_eq!(close_code(&mut desktop, WAIT).await, Some(4000));
    let mut again = relay.desktop("alice", peer(1)).await;
    assert_eq!(close_code(&mut again, WAIT).await, Some(4000));
}

#[tokio::test]
async fn the_tunnel_cap_refuses_one_more_socket() {
    let relay = Relay::with_limits(RelayLimits {
        max_tunnels: 1,
        ..RelayLimits::default()
    })
    .await;
    let first = relay.desktop("alice", peer(1)).await;
    let refused = relay
        .connect_desktop(&desktop_token("alice"), Some(peer(5)))
        .await;
    assert_eq!(refused_status(refused), 503);
    drop(first);
    tokio::time::sleep(Duration::from_millis(200)).await;
    drop(relay.desktop("alice", peer(5)).await);
}

#[tokio::test]
async fn a_second_socket_for_the_same_desktop_replaces_the_first() {
    let relay = Relay::start().await;
    let mut first = relay.desktop("alice", peer(1)).await;
    let mut second = relay.desktop("alice", peer(1)).await;
    assert_eq!(close_code(&mut first, WAIT).await, Some(4001));
    let (_, credential) = relay.pair_phone("alice", peer(2)).await;
    let mut phone = relay.phone(&credential).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    send(&mut phone, envelope(peer(1), peer(2), 0, b"to the new one")).await;
    assert!(recv(&mut second, WAIT).await.is_some());
}

fn unthrottled() -> RateLimit {
    RateLimit {
        messages_per_second: 1e6,
        message_burst: 1e6,
        bytes_per_second: 1e12,
        byte_burst: 1e12,
    }
}

#[tokio::test]
async fn a_phone_that_stops_reading_is_cut_its_permit_returns_and_nothing_after_is_lost() {
    let limits = RelayLimits {
        rate: unthrottled(),
        devices_per_user: 2,
        outbound_queue: 4,
        heartbeat_interval: Duration::from_millis(500),
        heartbeat_timeout: Duration::from_secs(1),
        ring: dynwork_relay::config::RingLimits {
            envelopes: 1_000,
            bytes: 128 * 1024 * 1024,
            ttl: Duration::from_secs(60),
        },
        ..RelayLimits::default()
    };
    let relay = Relay::with_limits(limits).await;
    let mut desktop = relay.desktop("alice", peer(1)).await;
    let (device, credential) = relay.pair_phone("alice", peer(2)).await;
    let device: dynwork_relay::ids::DeviceId = device.parse().expect("a device id");
    let silent = relay.phone(&credential).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    let last = 300u32;
    for seq in 0..=last {
        send(
            &mut desktop,
            envelope(peer(2), peer(1), seq, &vec![9; MAX_CIPHERTEXT]),
        )
        .await;
    }
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    while (relay.state.hub.is_online(device)
        || relay.state.tunnels.available_permits() < limits.max_tunnels - 1)
        && tokio::time::Instant::now() < deadline
    {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(!relay.state.hub.is_online(device), "the stuck phone is cut");
    assert!(
        relay.state.tunnels.available_permits() >= limits.max_tunnels - 1,
        "its permit returns"
    );
    drop(silent);
    let mut back = relay.phone(&credential).await;
    let mut seqs = Vec::new();
    while let Some(got) = recv(&mut back, Duration::from_secs(2)).await {
        seqs.push(u32::from_be_bytes(got[33..37].try_into().expect("a seq")));
        if seqs.last() == Some(&last) {
            break;
        }
    }
    assert_eq!(
        seqs.last(),
        Some(&last),
        "the newest envelope survives the cut"
    );
    let first = seqs[0];
    assert_eq!(
        seqs,
        (first..=last).collect::<Vec<_>>(),
        "a contiguous, ordered tail: only what reached the dead socket is lost"
    );
}

#[tokio::test]
async fn envelopes_sent_while_a_phone_is_closing_reach_it_after_it_reconnects() {
    let relay = Relay::with_limits(RelayLimits {
        session_lifetime: Duration::from_millis(400),
        ..RelayLimits::default()
    })
    .await;
    let (_, credential) = relay.pair_phone("alice", peer(2)).await;
    let mut phone = relay.phone(&credential).await;
    tokio::time::sleep(Duration::from_millis(600)).await;
    let mut desktop = relay.desktop("alice", peer(1)).await;
    for seq in 0..3u32 {
        send(
            &mut desktop,
            envelope(peer(2), peer(1), seq, b"during the close"),
        )
        .await;
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(close_code(&mut phone, WAIT).await, Some(4000));
    let mut back = relay.phone(&credential).await;
    for seq in 0..3u32 {
        let got = recv(&mut back, WAIT).await.expect("a held envelope");
        assert_eq!(got[33..37], seq.to_be_bytes());
    }
}

#[tokio::test]
async fn envelopes_of_an_unknown_version_count_against_the_rate() {
    let relay = Relay::with_limits(RelayLimits {
        rate: RateLimit {
            messages_per_second: 1.0,
            message_burst: 5.0,
            ..RateLimit::default()
        },
        ..RelayLimits::default()
    })
    .await;
    let mut desktop = relay.desktop("alice", peer(1)).await;
    for seq in 0..10u32 {
        let mut unknown = envelope(peer(2), peer(1), seq, b"v9");
        unknown[0] = 9;
        send(&mut desktop, unknown).await;
    }
    assert_eq!(close_code(&mut desktop, WAIT).await, Some(1008));
}
