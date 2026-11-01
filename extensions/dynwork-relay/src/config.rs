//! The relay's limits. The defaults fit one replica behind Envoy's 3600 s
//! route timeout and core's `server.max_in_flight`.

use std::time::Duration;

use crate::envelope::MAX_ENVELOPE;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RateLimit {
    pub messages_per_second: f64,
    pub message_burst: f64,
    pub bytes_per_second: f64,
    pub byte_burst: f64,
}

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            messages_per_second: 500.0,
            message_burst: 1_000.0,
            bytes_per_second: 8.0 * 1024.0 * 1024.0,
            byte_burst: 16.0 * 1024.0 * 1024.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RingLimits {
    pub envelopes: usize,
    pub bytes: usize,
    pub ttl: Duration,
}

impl Default for RingLimits {
    fn default() -> Self {
        Self {
            envelopes: 256,
            bytes: 4 * 1024 * 1024,
            ttl: Duration::from_secs(120),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RelayLimits {
    pub devices_per_user: u32,
    pub open_pairings_per_user: u32,
    pub pairing_ttl: Duration,
    pub max_envelope: usize,
    pub max_tunnels: usize,
    pub outbound_queue: usize,
    pub rate: RateLimit,
    pub heartbeat_interval: Duration,
    pub heartbeat_timeout: Duration,
    pub session_lifetime: Duration,
    pub ring: RingLimits,
    pub revocation_poll: Duration,
}

impl Default for RelayLimits {
    fn default() -> Self {
        Self {
            devices_per_user: 10,
            open_pairings_per_user: 5,
            pairing_ttl: Duration::from_secs(300),
            max_envelope: MAX_ENVELOPE,
            max_tunnels: 512,
            outbound_queue: 512,
            rate: RateLimit::default(),
            heartbeat_interval: Duration::from_secs(20),
            heartbeat_timeout: Duration::from_secs(60),
            session_lifetime: Duration::from_mins(50),
            ring: RingLimits::default(),
            revocation_poll: Duration::from_secs(2),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum LimitsError {
    #[error("the outbound queue must hold a presence message for every device of a user")]
    QueueSmallerThanDevices,
    #[error("the heartbeat timeout must exceed its interval")]
    HeartbeatTimeout,
    #[error("an envelope limit above the protocol maximum")]
    EnvelopeLimit,
    #[error("a zero limit")]
    Zero,
}

impl RelayLimits {
    pub fn validate(&self) -> Result<(), LimitsError> {
        if self.devices_per_user == 0 || self.max_tunnels == 0 || self.outbound_queue == 0 {
            return Err(LimitsError::Zero);
        }
        if self.outbound_queue < self.devices_per_user as usize {
            return Err(LimitsError::QueueSmallerThanDevices);
        }
        if self.heartbeat_timeout <= self.heartbeat_interval {
            return Err(LimitsError::HeartbeatTimeout);
        }
        if self.max_envelope > MAX_ENVELOPE {
            return Err(LimitsError::EnvelopeLimit);
        }
        Ok(())
    }
}
