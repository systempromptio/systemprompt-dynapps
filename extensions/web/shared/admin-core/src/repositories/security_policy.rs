//! `services/web/config/security.yaml` — the credential lifetime policy.
//!
//! One knob today: `max_pat_lifetime_days`, the longest a personal access
//! token may live from issuance. The bound is applied by [`bound_pat_expiry`],
//! which is pure so the arithmetic is pinned by a unit test, and read once at
//! the admin composition root into `ManagedState` because the file is
//! implementation configuration shipped with the image, not something the
//! console edits. An unparseable file is logged and served as the default (the
//! safe direction). No requested expiry means the maximum, never "forever";
//! more than the maximum is refused, not clamped.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;

pub const SECURITY_FILE: &str = "web/config/security.yaml";
pub const DEFAULT_MAX_PAT_LIFETIME_DAYS: u32 = 90;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SecurityPolicy {
    #[serde(default = "default_max_pat_lifetime_days")]
    pub max_pat_lifetime_days: u32,
}

const fn default_max_pat_lifetime_days() -> u32 {
    DEFAULT_MAX_PAT_LIFETIME_DAYS
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            max_pat_lifetime_days: DEFAULT_MAX_PAT_LIFETIME_DAYS,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SecurityDoc {
    #[serde(default)]
    security: SecurityPolicy,
}

#[derive(Debug, thiserror::Error)]
pub enum SecurityPolicyError {
    #[error("{path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("security policy does not parse: {0}")]
    Parse(#[source] serde_yaml::Error),
    #[error("security.max_pat_lifetime_days must be at least 1")]
    ZeroLifetime,
}

impl SecurityPolicy {
    #[must_use]
    pub fn load_or_default(services_dir: &Path) -> Self {
        match Self::load(services_dir) {
            Ok(policy) => policy,
            Err(error) => {
                tracing::error!(error = %error, file = SECURITY_FILE, "security policy unreadable; using defaults");
                Self::default()
            },
        }
    }

    pub fn load(services_dir: &Path) -> Result<Self, SecurityPolicyError> {
        let path = services_dir.join(SECURITY_FILE);
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(source) => return Err(SecurityPolicyError::Read { path, source }),
        };
        Self::from_yaml(&text)
    }

    pub fn from_yaml(text: &str) -> Result<Self, SecurityPolicyError> {
        let doc: SecurityDoc = serde_yaml::from_str(text).map_err(SecurityPolicyError::Parse)?;
        if doc.security.max_pat_lifetime_days == 0 {
            return Err(SecurityPolicyError::ZeroLifetime);
        }
        Ok(doc.security)
    }

    #[must_use]
    pub fn max_pat_lifetime(self) -> Duration {
        Duration::days(i64::from(self.max_pat_lifetime_days))
    }
}

/// Why a requested token expiry was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatExpiryError {
    InPast,
    BeyondPolicy { max_days: u32 },
}

impl std::fmt::Display for PatExpiryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InPast => f.write_str("expires_at must be in the future"),
            Self::BeyondPolicy { max_days } => write!(
                f,
                "expires_at exceeds the maximum token lifetime of {max_days} days"
            ),
        }
    }
}

impl std::error::Error for PatExpiryError {}

pub fn bound_pat_expiry(
    requested: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
    policy: SecurityPolicy,
) -> Result<DateTime<Utc>, PatExpiryError> {
    let ceiling = now + policy.max_pat_lifetime();
    match requested {
        None => Ok(ceiling),
        Some(at) if at <= now => Err(PatExpiryError::InPast),
        Some(at) if at > ceiling => Err(PatExpiryError::BeyondPolicy {
            max_days: policy.max_pat_lifetime_days,
        }),
        Some(at) => Ok(at),
    }
}
