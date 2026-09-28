//! Presence is one rule applied to a bridge heartbeat. It lives beside the
//! bridge tables rather than in any page so the count on the devices KPI
//! tile, the badge on the row and the user page can never disagree about
//! what online or stale means.

use chrono::{DateTime, Utc};

use crate::repositories::liveness::liveness_state;

pub const STALE_AFTER_DAYS: i64 = 7;

// Why: the desktop bridge's own `HEARTBEAT_INTERVAL` is 30 s; this must track
// it, since "online now" is a beat inside two of those intervals.
pub const BRIDGE_HEARTBEAT_SECS: i64 = 30;

/// What a bridge's last heartbeat says: beating now, quiet for less than a
/// week, or silent longer than that.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgePresence {
    Online,
    Idle,
    Stale,
}

impl BridgePresence {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Online => "Online",
            Self::Idle => "Idle",
            Self::Stale => "Stale",
        }
    }

    #[must_use]
    pub const fn tone(self) -> &'static str {
        match self {
            Self::Online => "ok",
            Self::Idle => "muted",
            Self::Stale => "warn",
        }
    }

    #[must_use]
    pub const fn is_online(self) -> bool {
        matches!(self, Self::Online)
    }
}

#[must_use]
pub fn bridge_presence(now: DateTime<Utc>, last_heartbeat: DateTime<Utc>) -> BridgePresence {
    if liveness_state(now, Some(last_heartbeat), BRIDGE_HEARTBEAT_SECS).is_alive() {
        BridgePresence::Online
    } else if (now - last_heartbeat).num_days() < STALE_AFTER_DAYS {
        BridgePresence::Idle
    } else {
        BridgePresence::Stale
    }
}
