use std::time::Duration;

use dynwork_relay::envelope::{Presence, presence};

use crate::support::{Relay, close_code, envelope, peer, recv, recv_presence, send};

const WAIT: Duration = Duration::from_secs(5);

#[tokio::test]
async fn a_phone_and_its_desktop_exchange_envelopes_with_the_sender_stamped() {
    let relay = Relay::start().await;
    let mut desktop = relay.desktop("alice", peer(1)).await;
    let (_, credential) = relay.pair_phone("alice", peer(2)).await;
    let mut phone = relay.phone(&credential).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    send(&mut phone, envelope(peer(1), peer(0xEE), 0, b"hello")).await;
    let at_desktop = recv(&mut desktop, WAIT)
        .await
        .expect("the desktop receives");
    assert_eq!(at_desktop, envelope(peer(1), peer(2), 0, b"hello"));

    send(&mut desktop, envelope(peer(2), peer(1), 0, b"world")).await;
    let at_phone = recv(&mut phone, WAIT).await.expect("the phone receives");
    assert_eq!(at_phone, envelope(peer(2), peer(1), 0, b"world"));
}

#[tokio::test]
async fn envelopes_between_two_peers_keep_their_order() {
    let relay = Relay::start().await;
    let mut desktop = relay.desktop("alice", peer(1)).await;
    let (_, credential) = relay.pair_phone("alice", peer(2)).await;
    let mut phone = relay.phone(&credential).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    for seq in 0..300u32 {
        send(
            &mut phone,
            envelope(peer(1), peer(2), seq, &seq.to_be_bytes()),
        )
        .await;
    }
    for seq in 0..300u32 {
        let got = recv(&mut desktop, WAIT).await.expect("an envelope");
        assert_eq!(got[33..37], seq.to_be_bytes());
    }
}

#[tokio::test]
async fn another_users_phone_cannot_reach_a_desktop() {
    let relay = Relay::start().await;
    let mut desktop = relay.desktop("alice", peer(1)).await;
    let (_, credential) = relay.pair_phone("mallory", peer(3)).await;
    let mut intruder = relay.phone(&credential).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    send(&mut intruder, envelope(peer(1), peer(3), 0, b"let me in")).await;
    assert_eq!(recv(&mut desktop, Duration::from_millis(400)).await, None);
    send(&mut intruder, envelope(peer(1), peer(3), 1, b"still here")).await;
    assert_eq!(
        close_code(&mut intruder, Duration::from_millis(200)).await,
        None
    );
}

#[tokio::test]
async fn a_desktop_cannot_claim_another_users_desktop_peer() {
    let relay = Relay::start().await;
    let mut alice = relay.desktop("alice", peer(1)).await;
    let mut mallory = relay.desktop("mallory", peer(1)).await;
    let (_, credential) = relay.pair_phone("alice", peer(2)).await;
    let mut phone = relay.phone(&credential).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    send(&mut phone, envelope(peer(1), peer(2), 0, b"for alice")).await;
    assert!(recv(&mut alice, WAIT).await.is_some());
    assert_eq!(recv(&mut mallory, Duration::from_millis(300)).await, None);
}

#[tokio::test]
async fn what_a_desktop_sends_an_offline_phone_is_delivered_when_it_returns() {
    let relay = Relay::start().await;
    let mut desktop = relay.desktop("alice", peer(1)).await;
    let (_, credential) = relay.pair_phone("alice", peer(2)).await;
    let phone = relay.phone(&credential).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    drop(phone);
    tokio::time::sleep(Duration::from_millis(100)).await;
    for seq in 0..3u32 {
        send(&mut desktop, envelope(peer(2), peer(1), seq, b"while away")).await;
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        relay
            .state
            .hub
            .held(&crate::support::user("alice"), peer(1)),
        3
    );
    let mut back = relay.phone(&credential).await;
    for seq in 0..3u32 {
        let got = recv(&mut back, WAIT).await.expect("a held envelope");
        assert_eq!(got[33..37], seq.to_be_bytes());
    }
}

#[tokio::test]
async fn an_envelope_of_an_unknown_version_is_dropped_without_closing() {
    let relay = Relay::start().await;
    let mut desktop = relay.desktop("alice", peer(1)).await;
    let (_, credential) = relay.pair_phone("alice", peer(2)).await;
    let mut phone = relay.phone(&credential).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    let mut wrong = envelope(peer(1), peer(2), 0, b"v2");
    wrong[0] = 2;
    send(&mut phone, wrong).await;
    send(&mut phone, envelope(peer(1), peer(2), 0, b"v1")).await;
    let got = recv(&mut desktop, WAIT).await.expect("the v1 envelope");
    assert_eq!(&got[37..], b"v1");
}

#[tokio::test]
async fn a_desktop_is_told_when_a_phone_connects_and_when_it_goes() {
    let relay = Relay::start().await;
    let mut desktop = relay.desktop("alice", peer(1)).await;
    let (_, credential) = relay.pair_phone("alice", peer(2)).await;
    let mut phone = relay.phone(&credential).await;
    let connected = recv_presence(&mut desktop, WAIT).await.expect("connected");
    assert_eq!(connected, presence(peer(1), peer(2), Presence::Connected));
    send(&mut phone, envelope(peer(1), peer(2), 0, b"last words")).await;
    drop(phone);
    let last = recv(&mut desktop, WAIT)
        .await
        .expect("the envelope before the departure");
    assert_eq!(&last[37..], b"last words");
    let gone = recv_presence(&mut desktop, WAIT)
        .await
        .expect("disconnected");
    assert_eq!(gone, presence(peer(1), peer(2), Presence::Disconnected));
}

#[tokio::test]
async fn a_relay_presence_message_sent_by_a_client_is_dropped() {
    let relay = Relay::start().await;
    let mut desktop = relay.desktop("alice", peer(1)).await;
    let (_, credential) = relay.pair_phone("alice", peer(2)).await;
    let mut phone = relay.phone(&credential).await;
    assert!(recv_presence(&mut desktop, WAIT).await.is_some());
    send(
        &mut phone,
        presence(peer(1), peer(9), Presence::Disconnected),
    )
    .await;
    assert_eq!(
        recv_presence(&mut desktop, Duration::from_millis(300)).await,
        None
    );
}
