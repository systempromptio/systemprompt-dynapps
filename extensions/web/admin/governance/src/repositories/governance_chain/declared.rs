//! The declaration: `services/governance/config.yaml` as core parses it.
//!
//! Parsed through core's own `GovernanceConfig::parse`, so a file the
//! console would accept is a file the boot would accept — an unknown mode
//! or a bad secret pattern is refused here with core's message. Hashed on
//! the parsed chain so a comment edit is not a drift.

use std::path::Path;

use sha2::{Digest, Sha256};
use systemprompt::governance::policy::GovernanceConfig;

use systemprompt_web_admin_core::error::{AdminError, AdminResult};

pub const GOVERNANCE_FILE: &str = "governance/config.yaml";

#[derive(Debug, Clone)]
pub struct DeclaredPolicy {
    pub id: String,
    pub enabled: bool,
    pub mode: String,
    pub entry: serde_yaml::Value,
}

#[derive(Debug, Clone)]
pub struct DeclaredChain {
    pub enabled: bool,
    pub mode: String,
    pub policies: Vec<DeclaredPolicy>,
}

impl Default for DeclaredChain {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: "enforce".to_owned(),
            policies: Vec::new(),
        }
    }
}

// Why: `serde_json` preserves the operator's YAML key order, so a plain dump
// would call a reordered mapping drift; RFC 8785 (`serde_jcs`) sorts keys.
// JSON: variable-shape governance policy entry
#[must_use]
pub fn entry_fingerprint(entry: &serde_yaml::Value) -> String {
    serde_json::to_value(entry)
        .ok()
        .and_then(|v| serde_jcs::to_string(&v).ok())
        .unwrap_or_default()
}

impl DeclaredChain {
    #[must_use]
    pub fn declared_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("chain\t{}\t{}\n", self.enabled, self.mode).as_bytes());
        for (i, p) in self.policies.iter().enumerate() {
            hasher.update(
                format!(
                    "policy\t{i}\t{}\t{}\t{}\t{}\n",
                    p.id,
                    p.enabled,
                    p.mode,
                    entry_fingerprint(&p.entry)
                )
                .as_bytes(),
            );
        }
        format!("{:x}", hasher.finalize())
    }

    #[must_use]
    pub fn find(&self, id: &str) -> Option<&DeclaredPolicy> {
        self.policies.iter().find(|p| p.id == id)
    }
}

// Why: core keeps the whole entry as `params`, id and switches included; the
// switches are hashed as fields, so an export spelling out an inherited mode
// must not read as a different declaration.
fn params_only(mut entry: serde_yaml::Value) -> serde_yaml::Value {
    if let Some(map) = entry.as_mapping_mut() {
        for key in ["id", "enabled", "mode"] {
            map.remove(serde_yaml::Value::from(key));
        }
    }
    entry
}

#[derive(Debug, thiserror::Error)]
pub enum DeclaredChainError {
    #[error("governance/config.yaml is invalid: {0}")]
    Config(#[from] systemprompt::governance::policy::GovernanceConfigError),
    #[error("governance.policies repeats id `{id}`")]
    DuplicatePolicy { id: String },
}

pub fn parse_declared_chain(yaml: &str) -> Result<DeclaredChain, DeclaredChainError> {
    let cfg = GovernanceConfig::parse(yaml)?;
    let mut policies = Vec::with_capacity(cfg.policies.len());
    for p in cfg.policies {
        if policies
            .iter()
            .any(|d: &DeclaredPolicy| d.id == p.id.as_str())
        {
            return Err(DeclaredChainError::DuplicatePolicy {
                id: p.id.to_string(),
            });
        }
        policies.push(DeclaredPolicy {
            id: p.id.to_string(),
            enabled: p.enabled,
            mode: p.mode.as_str().to_owned(),
            entry: params_only(p.params),
        });
    }
    Ok(DeclaredChain {
        enabled: cfg.enabled,
        mode: cfg.mode.as_str().to_owned(),
        policies,
    })
}

pub fn load_declared_chain(services_path: &Path) -> AdminResult<DeclaredChain> {
    let path = services_path.join(GOVERNANCE_FILE);
    let yaml = std::fs::read_to_string(&path)
        .map_err(|e| AdminError::invalid("governance/config.yaml could not be read", e))?;
    parse_declared_chain(&yaml)
        .map_err(|e| AdminError::invalid("governance/config.yaml could not be parsed", e))
}

pub fn declared_chain_now() -> AdminResult<DeclaredChain> {
    load_declared_chain(&crate::repositories::gateway_policies::declared::services_root()?)
}
