//! Whose identity a run's credentials are issued to: the one resolver every
//! launch goes through.
//!
//! The agent's `run_as` wins over the farm's when it sets anything. A mode
//! of `user` is the launcher, `client` is the run's client's service
//! account, and `service_account` is the account the sync resolved. A row
//! with no mode predates modes: its account when one is set, else the
//! launcher.

use systemprompt::identifiers::UserId;

use crate::config::RunAsMode;
use crate::error::RunAsRefusal;
use crate::types::ClientId;

/// One level's stored `run_as`: the mode column and the resolved account.
#[derive(Debug, Clone, Copy, Default)]
pub struct RunAsSetting<'a> {
    pub mode: Option<RunAsMode>,
    pub account: Option<&'a UserId>,
}

impl<'a> RunAsSetting<'a> {
    #[must_use]
    pub fn from_row(mode: Option<&str>, account: Option<&'a UserId>) -> Self {
        Self {
            mode: mode.and_then(RunAsMode::parse),
            account,
        }
    }

    const fn is_set(&self) -> bool {
        self.mode.is_some() || self.account.is_some()
    }

    const fn effective_mode(&self) -> RunAsMode {
        match (self.mode, self.account) {
            (Some(mode), _) => mode,
            (None, Some(_)) => RunAsMode::ServiceAccount,
            (None, None) => RunAsMode::User,
        }
    }
}

/// The run's client and the service account it declares, if any.
#[derive(Debug, Clone, Copy)]
pub struct RunAsClient<'a> {
    pub id: &'a ClientId,
    pub account: Option<&'a UserId>,
}

#[must_use]
pub const fn effective_mode(agent: RunAsSetting<'_>, farm: RunAsSetting<'_>) -> RunAsMode {
    let chosen = if agent.is_set() { agent } else { farm };
    chosen.effective_mode()
}

pub fn credential_user(
    agent: RunAsSetting<'_>,
    farm: RunAsSetting<'_>,
    client: Option<RunAsClient<'_>>,
    launcher: &UserId,
) -> Result<UserId, RunAsRefusal> {
    let chosen = if agent.is_set() { agent } else { farm };
    match chosen.effective_mode() {
        RunAsMode::User => Ok(launcher.clone()),
        RunAsMode::Client => {
            let client = client.ok_or(RunAsRefusal::NoClient)?;
            client
                .account
                .cloned()
                .ok_or_else(|| RunAsRefusal::ClientHasNoAccount {
                    client: client.id.as_str().to_owned(),
                })
        },
        RunAsMode::ServiceAccount => chosen
            .account
            .cloned()
            .ok_or(RunAsRefusal::NoServiceAccount),
    }
}
