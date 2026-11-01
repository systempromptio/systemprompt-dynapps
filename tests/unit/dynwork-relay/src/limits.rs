use std::time::Duration;

use dynwork_relay::RelayLimits;
use dynwork_relay::config::LimitsError;
use dynwork_relay::envelope::MAX_ENVELOPE;

#[test]
fn the_defaults_are_valid_and_close_under_envoys_timeout() {
    let limits = RelayLimits::default();
    assert_eq!(limits.validate(), Ok(()));
    assert_eq!(limits.session_lifetime, Duration::from_mins(50));
    assert!(limits.session_lifetime < Duration::from_secs(3_600));
    assert_eq!(limits.pairing_ttl, Duration::from_secs(300));
    assert_eq!(limits.max_envelope, MAX_ENVELOPE);
    assert!(limits.max_tunnels <= 512);
}

#[test]
fn inconsistent_limits_are_refused() {
    let queue = RelayLimits {
        outbound_queue: 4,
        ..RelayLimits::default()
    };
    assert_eq!(queue.validate(), Err(LimitsError::QueueSmallerThanDevices));
    let heartbeat = RelayLimits {
        heartbeat_timeout: Duration::from_secs(1),
        ..RelayLimits::default()
    };
    assert_eq!(heartbeat.validate(), Err(LimitsError::HeartbeatTimeout));
    let envelope = RelayLimits {
        max_envelope: MAX_ENVELOPE + 1,
        ..RelayLimits::default()
    };
    assert_eq!(envelope.validate(), Err(LimitsError::EnvelopeLimit));
}
