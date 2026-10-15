//! Pure decisions behind token resolution: whether a request may use the
//! saved grant at all, and what the grant needs — refresh, verification,
//! re-sealing — before its header is handed out.
//!
//! A probe (Test connection) is the user asking, so it bypasses the outage
//! hold and the verified-at gate; a saved OAuth grant is otherwise unusable
//! until one probe has succeeded.

use super::Grant;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

pub const REFRESH_LEEWAY_SECONDS: i64 = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Admission {
    Allowed,
    OutageHold,
    Unverified,
}

impl Admission {
    pub fn require(self) -> AdminResult<()> {
        match self {
            Self::Allowed => Ok(()),
            Self::OutageHold => Err(AdminError::Unavailable(
                "Connector provider recently unavailable; retry shortly".into(),
            )),
            Self::Unverified => Err(AdminError::Forbidden(
                "Test the connection on the Connectors page before using this MCP".into(),
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccessMode {
    pub probe: bool,
}

impl AccessMode {
    #[must_use]
    pub const fn admission(self, outage_hold: bool, verified: bool) -> Admission {
        if self.probe {
            Admission::Allowed
        } else if outage_hold {
            Admission::OutageHold
        } else if verified {
            Admission::Allowed
        } else {
            Admission::Unverified
        }
    }

    #[must_use]
    pub const fn verifies(self) -> bool {
        self.probe
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenPlan {
    pub refresh: bool,
    pub verify: bool,
}

impl TokenPlan {
    #[must_use]
    pub fn for_grant(grant: &Grant, mode: AccessMode, now: i64) -> Self {
        Self {
            refresh: needs_refresh(grant, now),
            verify: mode.verifies(),
        }
    }

    #[must_use]
    pub const fn reseals(self) -> bool {
        self.refresh || self.verify
    }
}

#[must_use]
pub fn needs_refresh(grant: &Grant, now: i64) -> bool {
    grant.auth_method == "oauth" && grant.expires_at <= now + REFRESH_LEEWAY_SECONDS
}
