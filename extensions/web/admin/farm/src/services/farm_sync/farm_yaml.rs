//! `farm.yaml`: the farm manifest naming its marketplace, the defaults its
//! agents fall back to, and the exact set of agents it ships.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::validate::{PERMISSION_MODES, definition_error, is_farm_id, is_identifier, require};
use crate::error::FarmResult;

pub const FARM_MANIFEST: &str = "farm.yaml";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestDefaults {
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub max_turns: Option<u32>,
    #[serde(default)]
    pub max_budget_usd: Option<f64>,
    #[serde(default)]
    pub permission_mode: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FarmManifest {
    pub schema: u32,
    pub id: String,
    pub marketplace: String,
    #[serde(default)]
    pub defaults: ManifestDefaults,
    pub agents: Vec<String>,
}

pub fn parse_farm_manifest(text: &str) -> FarmResult<FarmManifest> {
    let path = FARM_MANIFEST;
    let manifest: FarmManifest =
        serde_yaml::from_str(text).map_err(|e| definition_error(path, e))?;
    require(
        manifest.schema == 1,
        path,
        format!("schema {} is not 1", manifest.schema),
    )?;
    require(
        is_farm_id(&manifest.id),
        path,
        format!(
            "id {:?} does not match ^[a-z][a-z0-9-]{{2,63}}$",
            manifest.id
        ),
    )?;
    require(
        !manifest.marketplace.trim().is_empty(),
        path,
        "marketplace is empty",
    )?;
    require(
        !manifest.agents.is_empty(),
        path,
        "agents must be a non-empty list",
    )?;
    let defaults = &manifest.defaults;
    require(
        defaults.max_turns != Some(0),
        path,
        "defaults.max_turns must be > 0",
    )?;
    require(
        defaults
            .max_budget_usd
            .is_none_or(|b| b.is_finite() && b > 0.0),
        path,
        "defaults.max_budget_usd must be > 0",
    )?;
    if let Some(mode) = &defaults.permission_mode {
        require(
            PERMISSION_MODES.contains(&mode.as_str()),
            path,
            format!("defaults.permission_mode {mode:?} is not one of {PERMISSION_MODES:?}"),
        )?;
    }
    let mut seen = BTreeSet::new();
    for agent in &manifest.agents {
        require(
            is_identifier(agent),
            path,
            format!("agent id {agent:?} is invalid"),
        )?;
        require(
            seen.insert(agent.as_str()),
            path,
            format!("agent {agent:?} listed twice"),
        )?;
    }
    Ok(manifest)
}

pub fn check_agents_listed(
    manifest: &FarmManifest,
    agent_files: &BTreeSet<String>,
) -> FarmResult<()> {
    let listed: BTreeSet<String> = manifest.agents.iter().cloned().collect();
    if let Some(missing) = listed.difference(agent_files).next() {
        return Err(definition_error(
            FARM_MANIFEST,
            format!("agent {missing:?} is listed but agents/{missing}.md does not exist"),
        ));
    }
    if let Some(unlisted) = agent_files.difference(&listed).next() {
        return Err(definition_error(
            FARM_MANIFEST,
            format!("agents/{unlisted}.md exists but {unlisted:?} is not listed"),
        ));
    }
    Ok(())
}
