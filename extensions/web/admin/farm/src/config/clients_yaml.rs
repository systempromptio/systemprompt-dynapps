//! The client and per-agent blocks of `web/config/agent-farms.yaml`: who a
//! farm's runs are for, and which service account a single agent runs as
//! when it should not run as its farm's.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::error::{FarmError, FarmResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientDef {
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub odoo_partner_id: Option<i64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentOverride {
    #[serde(default)]
    pub run_as: Option<String>,
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
        if !seen.insert(client.slug.as_str()) {
            return Err(FarmError::Config(format!(
                "{file}: client {:?} is declared twice",
                client.slug
            )));
        }
    }
    Ok(())
}
