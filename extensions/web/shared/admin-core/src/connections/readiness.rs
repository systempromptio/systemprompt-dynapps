//! Whether a connection will carry a person's calls, and which sub-condition
//! failed when it will not.

use super::Connection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotReady {
    NotConfigured,
    NotEntitled,
    Status,
    Unverified,
}

impl NotReady {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotConfigured => "not_configured",
            Self::NotEntitled => "not_entitled",
            Self::Status => "status",
            Self::Unverified => "verified_at_null",
        }
    }
}

impl std::fmt::Display for NotReady {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Connection {
    pub fn readiness(&self) -> Result<(), NotReady> {
        if !self.configured {
            return Err(NotReady::NotConfigured);
        }
        if !self.requires_auth {
            return Ok(());
        }
        if !self.entitled {
            return Err(NotReady::NotEntitled);
        }
        if !matches!(
            self.status.as_str(),
            "connected" | "temporarily_unavailable"
        ) {
            return Err(NotReady::Status);
        }
        if self.verified_at.is_none() {
            return Err(NotReady::Unverified);
        }
        Ok(())
    }

    pub fn is_ready(&self) -> bool {
        self.readiness().is_ok()
    }
}
