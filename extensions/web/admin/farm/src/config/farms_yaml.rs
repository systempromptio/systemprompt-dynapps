//! Typed form of `web/config/agent-farms.yaml`, read the way the groups file
//! is: a missing file is no farms, an empty one is an empty document, and an
//! unknown key is a load error rather than a silently ignored setting.

use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

use super::clients_yaml::{AgentOverride, ClientDef, validate_clients};
use super::evals_yaml::EvalsConfig;
use crate::error::{FarmError, FarmResult};

pub const FARMS_FILE: &str = "web/config/agent-farms.yaml";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FarmsDoc {
    #[serde(default)]
    pub clients: Vec<ClientDef>,
    #[serde(default)]
    pub farms: Vec<FarmDef>,
    #[serde(default)]
    pub runner: Option<RunnerConfig>,
    #[serde(default)]
    pub evals: Option<EvalsConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FarmDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub marketplace: String,
    pub source: GitSource,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub run_as: Option<String>,
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub client: Option<String>,
    #[serde(default)]
    pub agents: BTreeMap<String, AgentOverride>,
    #[serde(default)]
    pub defaults: FarmDefaults,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitSource {
    pub github: String,
    #[serde(default = "default_ref")]
    pub r#ref: String,
    #[serde(default = "default_subdir")]
    pub subdir: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FarmDefaults {
    #[serde(default = "default_permission_mode")]
    pub permission_mode: String,
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,
    #[serde(default = "default_max_turns")]
    pub max_turns: u32,
    #[serde(default = "default_max_budget_usd")]
    pub max_budget_usd: f64,
    #[serde(default)]
    pub model: Option<String>,
}

impl Default for FarmDefaults {
    fn default() -> Self {
        Self {
            permission_mode: default_permission_mode(),
            timeout_seconds: default_timeout_seconds(),
            max_turns: default_max_turns(),
            max_budget_usd: default_max_budget_usd(),
            model: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunnerConfig {
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: usize,
    #[serde(default)]
    pub claude_bin: Option<String>,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            max_concurrent: default_max_concurrent(),
            claude_bin: None,
        }
    }
}

fn default_ref() -> String {
    "main".to_owned()
}

fn default_subdir() -> String {
    "farm".to_owned()
}

fn default_permission_mode() -> String {
    "acceptEdits".to_owned()
}

const fn default_timeout_seconds() -> u64 {
    1200
}

const fn default_max_turns() -> u32 {
    40
}

const fn default_max_budget_usd() -> f64 {
    5.0
}

const fn default_max_concurrent() -> usize {
    2
}

const PERMISSION_MODES: [&str; 4] = ["default", "acceptEdits", "plan", "bypassPermissions"];

pub(super) fn is_slug(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

impl FarmsDoc {
    pub fn validate(&self) -> FarmResult<()> {
        validate_clients(&self.clients, FARMS_FILE)?;
        let clients: BTreeSet<&str> = self.clients.iter().map(|c| c.slug.as_str()).collect();
        let mut seen = BTreeSet::new();
        for farm in &self.farms {
            if let Some(client) = farm.client.as_deref()
                && !clients.contains(client)
            {
                return Err(invalid(&format!(
                    "farm {:?}: client {client:?} is not declared under clients",
                    farm.id
                )));
            }
            if farm
                .agents
                .values()
                .any(|a| a.run_as.as_deref().is_some_and(|id| id.trim().is_empty()))
            {
                return Err(invalid(&format!(
                    "farm {:?}: an agent run_as is empty",
                    farm.id
                )));
            }
            if !is_slug(&farm.id) {
                return Err(invalid(&format!("farm id {:?} is not a slug", farm.id)));
            }
            if !seen.insert(farm.id.as_str()) {
                return Err(invalid(&format!("farm id {:?} is declared twice", farm.id)));
            }
            if !is_slug(&farm.marketplace) {
                return Err(invalid(&format!(
                    "farm {:?}: marketplace {:?} is not a slug",
                    farm.id, farm.marketplace
                )));
            }
            if farm
                .run_as
                .as_deref()
                .is_some_and(|id| id.trim().is_empty())
            {
                return Err(invalid(&format!("farm {:?}: run_as is empty", farm.id)));
            }
            farm.source.validate(&farm.id)?;
            farm.defaults.validate(&farm.id)?;
        }
        if self.runner.as_ref().is_some_and(|r| r.max_concurrent == 0) {
            return Err(invalid("runner.max_concurrent must be at least 1"));
        }
        if let Some(evals) = &self.evals {
            evals.validate(FARMS_FILE)?;
        }
        Ok(())
    }
}

impl GitSource {
    fn validate(&self, farm: &str) -> FarmResult<()> {
        let mut parts = self.github.split('/');
        let well_formed = matches!(
            (parts.next(), parts.next(), parts.next()),
            (Some(owner), Some(repo), None) if !owner.is_empty() && !repo.is_empty()
        );
        if !well_formed {
            return Err(invalid(&format!(
                "farm {farm:?}: source.github {:?} is not owner/repo",
                self.github
            )));
        }
        if self.r#ref.trim().is_empty() {
            return Err(invalid(&format!("farm {farm:?}: source.ref is empty")));
        }
        if self.subdir.starts_with('/') || self.subdir.split('/').any(|seg| seg == "..") {
            return Err(invalid(&format!(
                "farm {farm:?}: source.subdir {:?} must stay inside the repository",
                self.subdir
            )));
        }
        Ok(())
    }
}

impl FarmDefaults {
    fn validate(&self, farm: &str) -> FarmResult<()> {
        if !PERMISSION_MODES.contains(&self.permission_mode.as_str()) {
            return Err(invalid(&format!(
                "farm {farm:?}: permission_mode {:?} is not one of {PERMISSION_MODES:?}",
                self.permission_mode
            )));
        }
        if self.timeout_seconds == 0 || self.max_turns == 0 {
            return Err(invalid(&format!(
                "farm {farm:?}: timeout_seconds and max_turns must be positive"
            )));
        }
        if !self.max_budget_usd.is_finite() || self.max_budget_usd <= 0.0 {
            return Err(invalid(&format!(
                "farm {farm:?}: max_budget_usd must be a positive number"
            )));
        }
        Ok(())
    }
}

fn invalid(message: &str) -> FarmError {
    FarmError::Config(format!("{FARMS_FILE}: {message}"))
}

pub fn parse_farms_doc(text: &str) -> FarmResult<FarmsDoc> {
    if text.trim().is_empty() {
        return Ok(FarmsDoc::default());
    }
    serde_yaml::from_str::<FarmsDoc>(text).map_err(|e| invalid(&e.to_string()))
}

pub async fn read_farms_doc(services_path: &Path) -> FarmResult<Option<FarmsDoc>> {
    let path = services_path.join(FARMS_FILE);
    match tokio::fs::read_to_string(&path).await {
        Ok(text) => {
            let doc = parse_farms_doc(&text)?;
            doc.validate()?;
            Ok(Some(doc))
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}
