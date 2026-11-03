//! How a tunnel is told to close, and the close codes it sends.

use std::sync::OnceLock;

use tokio_util::sync::CancellationToken;

/// Why the relay closes a socket. `Reconnect` and `Drain` are the reconnect
/// hints: the peer should dial again at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseReason {
    Reconnect,
    Drain,
    Revoked,
    Replaced,
    TooLarge,
    RateLimited,
    Malformed,
    BinaryOnly,
    Heartbeat,
    SlowConsumer,
}

impl CloseReason {
    pub const fn code(self) -> u16 {
        match self {
            Self::Reconnect => 4000,
            Self::Replaced => 4001,
            Self::Revoked => 4003,
            Self::Heartbeat => 4008,
            Self::Drain => 1012,
            Self::TooLarge => 1009,
            Self::RateLimited | Self::Malformed => 1008,
            Self::BinaryOnly => 1003,
            Self::SlowConsumer => 1013,
        }
    }

    pub const fn text(self) -> &'static str {
        match self {
            Self::Reconnect | Self::Drain => "reconnect",
            Self::Revoked => "revoked",
            Self::Replaced => "replaced",
            Self::TooLarge => "envelope too large",
            Self::RateLimited => "rate limit",
            Self::Malformed => "malformed envelope",
            Self::BinaryOnly => "binary only",
            Self::Heartbeat => "heartbeat timeout",
            Self::SlowConsumer => "slow consumer",
        }
    }
}

#[derive(Debug, Default)]
pub struct Control {
    cancel: CancellationToken,
    reason: OnceLock<CloseReason>,
}

impl Control {
    pub fn close(&self, reason: CloseReason) {
        self.reason.get_or_init(|| reason);
        self.cancel.cancel();
    }

    pub fn reason(&self) -> Option<CloseReason> {
        self.reason.get().copied()
    }

    pub async fn closed(&self) -> CloseReason {
        self.cancel.cancelled().await;
        self.reason().unwrap_or(CloseReason::Reconnect)
    }
}
