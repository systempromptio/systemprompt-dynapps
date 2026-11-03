//! Audit events of the relay and the mobile sign-in.
//!
//! `connect`, `disconnect`, `pair`, `revoke`, the pairing token's creation,
//! and the mobile sign-in's consent and sign-in. They carry
//! ids, kinds, counts and reasons only — never a token, a credential or a byte
//! of ciphertext.

use std::fmt::Debug;

use systemprompt::identifiers::UserId;

use crate::ids::{DeviceId, PairingId, PeerKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionStats {
    pub envelopes_in: u64,
    pub envelopes_out: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditEvent {
    PairingCreated {
        user: UserId,
        pairing: PairingId,
    },
    Pair {
        user: UserId,
        pairing: PairingId,
        device: DeviceId,
    },
    Connect {
        user: UserId,
        device: DeviceId,
        kind: PeerKind,
    },
    Disconnect {
        user: UserId,
        device: DeviceId,
        kind: PeerKind,
        reason: &'static str,
        stats: ConnectionStats,
    },
    Revoke {
        user: UserId,
        device: DeviceId,
    },
    MobileConsent {
        user: UserId,
        approved: bool,
    },
    MobileSignIn {
        user: UserId,
        device: DeviceId,
    },
}

impl AuditEvent {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::PairingCreated { .. } => "pairing_created",
            Self::Pair { .. } => "pair",
            Self::Connect { .. } => "connect",
            Self::Disconnect { .. } => "disconnect",
            Self::Revoke { .. } => "revoke",
            Self::MobileConsent { .. } => "mobile_consent",
            Self::MobileSignIn { .. } => "mobile_sign_in",
        }
    }
}

pub trait AuditSink: Send + Sync + Debug {
    fn record(&self, event: &AuditEvent);
}

/// Writes each event to the `dynwork_relay::audit` tracing target, which
/// core's log pipeline stores.
#[derive(Debug, Clone, Copy, Default)]
pub struct TracingAudit;

struct Fields<'a> {
    user: &'a UserId,
    device: Option<String>,
    pairing: Option<String>,
    kind: Option<&'static str>,
    reason: Option<&'static str>,
    stats: Option<ConnectionStats>,
}

impl<'a> Fields<'a> {
    fn of(event: &'a AuditEvent) -> Self {
        let mut fields = Self {
            user: match event {
                AuditEvent::PairingCreated { user, .. }
                | AuditEvent::Pair { user, .. }
                | AuditEvent::Connect { user, .. }
                | AuditEvent::Disconnect { user, .. }
                | AuditEvent::Revoke { user, .. }
                | AuditEvent::MobileConsent { user, .. }
                | AuditEvent::MobileSignIn { user, .. } => user,
            },
            device: None,
            pairing: None,
            kind: None,
            reason: None,
            stats: None,
        };
        match event {
            AuditEvent::PairingCreated { pairing, .. } => {
                fields.pairing = Some(pairing.to_string());
            },
            AuditEvent::Pair {
                pairing, device, ..
            } => {
                fields.pairing = Some(pairing.to_string());
                fields.device = Some(device.to_string());
            },
            AuditEvent::Connect { device, kind, .. } => {
                fields.device = Some(device.to_string());
                fields.kind = Some(kind.as_str());
            },
            AuditEvent::Disconnect {
                device,
                kind,
                reason,
                stats,
                ..
            } => {
                fields.device = Some(device.to_string());
                fields.kind = Some(kind.as_str());
                fields.reason = Some(reason);
                fields.stats = Some(*stats);
            },
            AuditEvent::Revoke { device, .. } | AuditEvent::MobileSignIn { device, .. } => {
                fields.device = Some(device.to_string());
            },
            AuditEvent::MobileConsent { approved, .. } => {
                fields.reason = Some(if *approved { "approved" } else { "denied" });
            },
        }
        fields
    }
}

impl AuditSink for TracingAudit {
    fn record(&self, event: &AuditEvent) {
        let fields = Fields::of(event);
        tracing::info!(
            target: "dynwork_relay::audit",
            event = event.name(),
            user_id = %fields.user,
            device_id = fields.device.as_deref(),
            pairing_id = fields.pairing.as_deref(),
            kind = fields.kind,
            reason = fields.reason,
            envelopes_in = fields.stats.map(|stats| stats.envelopes_in),
            envelopes_out = fields.stats.map(|stats| stats.envelopes_out),
            duration_ms = fields.stats.map(|stats| stats.duration_ms),
            "dynwork relay audit"
        );
    }
}
