//! `run_as` in agent-farms.yaml, on a farm and per agent: whose identity a
//! run's credentials are issued to.
//!
//! - `user`: the person who launched the run, spending their own connector
//!   grants (Odoo, DynBrain) through the gateway.
//! - `client`: the service account of the run's client (`clients[].
//!   service_account`), so every run for one client acts as that client.
//! - `service_account`: the named service account (`service_account:`).
//!
//! The legacy plain form `run_as: svc@example.com` still parses, as
//! `service_account` naming that account.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunAsMode {
    User,
    Client,
    ServiceAccount,
}

impl RunAsMode {
    pub const ALL: [Self; 3] = [Self::User, Self::Client, Self::ServiceAccount];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Client => "client",
            Self::ServiceAccount => "service_account",
        }
    }

    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|mode| mode.as_str() == raw)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RunAsRepr")]
pub struct RunAs {
    pub mode: RunAsMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_account: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RunAsBlock {
    mode: RunAsMode,
    #[serde(default)]
    service_account: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RunAsRepr {
    Legacy(String),
    Block(RunAsBlock),
}

impl TryFrom<RunAsRepr> for RunAs {
    type Error = String;

    fn try_from(repr: RunAsRepr) -> Result<Self, Self::Error> {
        Ok(match repr {
            RunAsRepr::Legacy(account) => Self::service_account(account),
            RunAsRepr::Block(block) => Self {
                mode: block.mode,
                service_account: block.service_account,
            },
        })
    }
}

impl RunAs {
    #[must_use]
    pub fn service_account(account: impl Into<String>) -> Self {
        Self {
            mode: RunAsMode::ServiceAccount,
            service_account: Some(account.into()),
        }
    }

    #[must_use]
    pub const fn of(mode: RunAsMode) -> Self {
        Self {
            mode,
            service_account: None,
        }
    }

    /// The account to resolve at sync, present only in `service_account` mode.
    #[must_use]
    pub fn account(&self) -> Option<&str> {
        match self.mode {
            RunAsMode::ServiceAccount => self.service_account.as_deref().map(str::trim),
            RunAsMode::User | RunAsMode::Client => None,
        }
    }

    pub(super) fn validate(&self, owner: &str) -> Result<(), String> {
        let named = self
            .service_account
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());
        match (self.mode, named, self.service_account.is_some()) {
            (RunAsMode::ServiceAccount, Some(_), _) => Ok(()),
            (RunAsMode::ServiceAccount, None, _) => Err(format!("{owner}: run_as is empty")),
            (_, _, true) => Err(format!(
                "{owner}: run_as.service_account is only read in mode service_account"
            )),
            _ => Ok(()),
        }
    }
}
