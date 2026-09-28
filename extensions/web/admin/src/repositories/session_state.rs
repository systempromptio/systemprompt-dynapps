//! Whether a client session is still going, decided from what it last did.
//!
//! A hook session states one fact about its own life: `SessionEnd` was seen
//! (`plugin_session_summaries.ended_at`, or `status = 'completed'`). Nothing
//! else it writes says "still running" — the status column is only ever
//! NULL or completed — so a session with no end is live or stale by how
//! recently it did anything from any vantage point: the last gateway
//! request or the last hook event. The window is the MCP liveness rule's,
//! so the session page and the MCP pages agree on what "recently" means.

use chrono::{DateTime, Utc};

use systemprompt_web_admin_core::repositories::liveness::{
    HEARTBEAT_INTERVAL_SECS, liveness_state,
};

/// One of three states every session badge reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Live,
    Stale,
    Completed,
}

impl SessionState {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::Stale => "stale",
            Self::Completed => "completed",
        }
    }

    #[must_use]
    pub const fn tone(self) -> &'static str {
        match self {
            Self::Live => "ok",
            Self::Stale => "warn",
            Self::Completed => "muted",
        }
    }
}

#[must_use]
pub fn session_state(
    now: DateTime<Utc>,
    completed: bool,
    last_activity_at: Option<DateTime<Utc>>,
) -> SessionState {
    if completed {
        return SessionState::Completed;
    }
    if liveness_state(now, last_activity_at, HEARTBEAT_INTERVAL_SECS).is_alive() {
        SessionState::Live
    } else {
        SessionState::Stale
    }
}
