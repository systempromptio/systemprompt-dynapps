//! Pure workflow transitions. Only validated predecessors and human receipts
//! unlock successors.

use crate::error::{RequirementsError, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use systemprompt::identifiers::UserId;
use uuid::Uuid;

pub const VALIDATOR_VERSION: &str = "requirements-v2.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorkflowId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorkflowProjectId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Context,
    Specification,
}

impl Phase {
    pub const ALL: [Self; 2] = [Self::Context, Self::Specification];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::Specification => "specification",
        }
    }

    pub fn accepts_path(self, path: &str) -> bool {
        path == format!("{}.md", self.name())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceSelection {
    pub source_type: String,
    pub source_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Destination {
    pub cloud_id: String,
    pub space_id: String,
    pub parent_id: String,
    pub title: String,
}

impl Destination {
    pub fn validate(&self) -> Result<()> {
        let shaped = [&self.cloud_id, &self.space_id, &self.parent_id, &self.title]
            .iter()
            .all(|s| !s.trim().is_empty() && s.len() <= 255 && !s.chars().any(char::is_control));
        if shaped {
            Ok(())
        } else {
            Err(RequirementsError::Validation(
                "Destination requires a cloud, space, parent and title".to_owned(),
            ))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub source_ref: String,
    pub uri: String,
    pub title: String,
    pub source_type: String,
    pub body: String,
    pub hash: String,
    pub version: String,
    pub indexed_hash: String,
    pub retrieval_method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validation {
    pub artifact_hash: String,
    pub dependency_hash: String,
    pub passed: bool,
    pub report: String,
    pub validator_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub files: BTreeMap<String, String>,
    pub hash: String,
    pub dependency_hash: String,
    pub validation: Option<Validation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Gate {
    Context,
    Publication,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub payload: String,
    pub key_id: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub gate: Gate,
    pub bundle_hash: String,
    pub receipt: Option<Receipt>,
    pub rejection: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Publication {
    pub page_id: String,
    pub uri: String,
    pub body_hash: String,
    pub bundle_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: WorkflowId,
    pub name: String,
    pub evidence: Vec<Evidence>,
    pub delivery_scope: String,
    pub destination: Option<Destination>,
    pub evidence_hash: String,
    pub validator_version: String,
    pub artifacts: BTreeMap<Phase, Artifact>,
    pub context_review: Option<Review>,
    pub publication_review: Option<Review>,
    pub publications: BTreeMap<String, Publication>,
}

/// A workflow as persisted: its optimistic-concurrency revision, the owner
/// whose evidence it froze, and the project it belongs to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredWorkflow {
    pub revision: i64,
    pub owner_id: UserId,
    pub project_id: WorkflowProjectId,
    pub workflow: Workflow,
}

pub fn digest<T: Serialize>(value: &T) -> Result<String> {
    Ok(hex::encode(Sha256::digest(serde_jcs::to_vec(value)?)))
}

pub fn valid_slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 80
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
