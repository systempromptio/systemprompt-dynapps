//! The declaration: `services/gateway/policies.yaml` as core parses it.
//!
//! Read from the composed services root so a kit that ships a policy file
//! declares through the same door, and hashed on the parsed content so a
//! comment or a whitespace edit is not a drift.

use sha2::{Digest, Sha256};
use systemprompt::config::ProfileBootstrap;
use systemprompt::gateway::{GATEWAY_POLICIES_FILE, GatewayPolicyConfig, GatewayPolicyEntry};
use systemprompt::loader::services_root::ServicesRootBootstrap;

use systemprompt_web_admin_core::error::AdminResult;

pub const POLICIES_FILE: &str = GATEWAY_POLICIES_FILE;

#[derive(Debug, thiserror::Error)]
pub enum DeclaredPoliciesError {
    #[error("policies.yaml does not parse: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("policies.yaml is invalid: {0}")]
    Invalid(#[from] systemprompt::gateway::GatewayPolicyError),
}

/// What the file declares, with each monthly window folded to its sentinel
/// so a declared hash never depends on the day it was read.
#[derive(Debug, Clone, Default)]
pub struct DeclaredPolicies {
    pub entries: Vec<GatewayPolicyEntry>,
}

impl DeclaredPolicies {
    #[must_use]
    pub fn declared_hash(&self) -> String {
        let mut hasher = Sha256::new();
        for e in &self.entries {
            // Why: discard-ok: infallible for a value that already round-tripped
            let spec = serde_json::to_string(&e.spec).unwrap_or_default();
            hasher.update(
                format!(
                    "policy\t{}\t{}\t{}\t{spec}\n",
                    e.name, e.enabled, e.priority
                )
                .as_bytes(),
            );
        }
        format!("{:x}", hasher.finalize())
    }

    #[must_use]
    pub fn find(&self, name: &str) -> Option<&GatewayPolicyEntry> {
        self.entries.iter().find(|e| e.name == name)
    }
}

pub fn parse_declared_policies(yaml: &str) -> Result<DeclaredPolicies, DeclaredPoliciesError> {
    let cfg: GatewayPolicyConfig = serde_yaml::from_str(yaml)?;
    cfg.validate()?;
    Ok(DeclaredPolicies {
        entries: cfg.policies.into_iter().collect(),
    })
}

pub fn services_root() -> AdminResult<std::path::PathBuf> {
    let profile = ProfileBootstrap::get()?;
    Ok(ServicesRootBootstrap::active_root_or(
        &profile.paths.services,
    ))
}
