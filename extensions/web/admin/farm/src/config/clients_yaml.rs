//! The client and per-agent blocks of `web/config/agent-farms.yaml`: who a
//! farm's runs are for, the service account a `run_as: {mode: client}` run
//! acts as for that client, and the `run_as` a single agent uses when it
//! should not use its farm's.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::run_as::RunAs;
use crate::error::{FarmError, FarmResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientDef {
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub service_account: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentOverride {
    #[serde(default)]
    pub run_as: Option<RunAs>,
}

pub(super) fn validate_clients(clients: &[ClientDef], file: &str) -> FarmResult<()> {
    let mut seen = BTreeSet::new();
    for client in clients {
        if !super::farms_yaml::is_slug(&client.slug) {
            return Err(FarmError::Config(format!(
                "{file}: client slug {:?} is not a slug",
                client.slug
            )));
        }
        if client.name.trim().is_empty() {
            return Err(FarmError::Config(format!(
                "{file}: client {:?} has no name",
                client.slug
            )));
        }
        if client
            .service_account
            .as_deref()
            .is_some_and(|id| id.trim().is_empty())
        {
            return Err(FarmError::Config(format!(
                "{file}: client {:?} has an empty service_account",
                client.slug
            )));
        }
        if !seen.insert(client.slug.as_str()) {
            return Err(FarmError::Config(format!(
                "{file}: client {:?} is declared twice",
                client.slug
            )));
        }
    }
    Ok(())
}
