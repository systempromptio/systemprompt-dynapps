use dynwork_relay::audit::{AuditEvent, AuditSink, ConnectionStats, TracingAudit};
use dynwork_relay::ids::{DeviceId, PairingId, PeerKind};
use systemprompt::identifiers::UserId;

#[test]
fn every_event_has_a_name_and_the_tracing_sink_takes_each() {
    let user = UserId::try_new("alice").expect("a user id");
    let device = DeviceId::generate();
    let pairing = PairingId::generate();
    let events = [
        AuditEvent::PairingCreated {
            user: user.clone(),
            pairing,
        },
        AuditEvent::Pair {
            user: user.clone(),
            pairing,
            device,
        },
        AuditEvent::Connect {
            user: user.clone(),
            device,
            kind: PeerKind::Phone,
        },
        AuditEvent::Disconnect {
            user: user.clone(),
            device,
            kind: PeerKind::Phone,
            reason: "revoked",
            stats: ConnectionStats {
                envelopes_in: 1,
                envelopes_out: 2,
                duration_ms: 3,
            },
        },
        AuditEvent::Revoke { user, device },
    ];
    let names: Vec<&str> = events.iter().map(AuditEvent::name).collect();
    assert_eq!(
        names,
        ["pairing_created", "pair", "connect", "disconnect", "revoke"]
    );
    for event in &events {
        TracingAudit.record(event);
    }
}
