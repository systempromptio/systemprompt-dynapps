use std::time::Duration;

use systemprompt::traits::DrainOutcome;

use crate::support::{Relay, close_code, peer};

#[tokio::test]
async fn shutdown_closes_every_tunnel_with_a_reconnect_hint_and_drains() {
    let relay = Relay::start().await;
    let mut desktop = relay.desktop("alice", peer(1)).await;
    let (_, credential) = relay.pair_phone("alice", peer(2)).await;
    let mut phone = relay.phone(&credential).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(relay.tasks.in_flight(), 2);
    let outcome = relay.tasks.shutdown(Duration::from_secs(5)).await;
    assert_eq!(outcome, DrainOutcome::Drained);
    assert_eq!(
        close_code(&mut desktop, Duration::from_secs(5)).await,
        Some(1012)
    );
    assert_eq!(
        close_code(&mut phone, Duration::from_secs(5)).await,
        Some(1012)
    );
    assert_eq!(relay.state.hub.tunnels(), 0);
}

#[tokio::test]
async fn ending_streams_closes_tunnels_with_a_reconnect_hint() {
    let relay = Relay::start().await;
    let mut desktop = relay.desktop("alice", peer(1)).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    relay.tasks.stream_drain().end_streams();
    assert_eq!(
        close_code(&mut desktop, Duration::from_secs(5)).await,
        Some(1012)
    );
}
