//! Pure workflow transitions. Only validated predecessors and human receipts
//! unlock successors.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use uuid::Uuid;

pub const RUBRIC_COMMIT: &str = "2209c7ecd294fe728141d6e4fe55c74d5cacc968";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorkflowId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Discovery,
    Design,
    Proposal,
    Grooming,
    Specs,
    Fsd,
}

impl Phase {
    pub const ALL: [Self; 6] = [
        Self::Discovery,
        Self::Design,
        Self::Proposal,
        Self::Grooming,
        Self::Specs,
        Self::Fsd,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Discovery => "discovery",
            Self::Design => "design",
            Self::Proposal => "proposal",
            Self::Grooming => "grooming",
            Self::Specs => "specs",
            Self::Fsd => "fsd",
        }
    }

    pub fn accepts_path(self, path: &str) -> bool {
        if path.contains("..") || path.contains('\\') || path.starts_with('/') {
            return false;
        }
        let parts: Vec<_> = path.split('/').collect();
        match self {
            Self::Specs => {
                parts.len() == 3
                    && parts[0] == "specs"
                    && valid_slug(parts[1])
                    && parts[2] == "spec.md"
            },
            Self::Fsd => {
                parts.len() == 2
                    && parts[0] == "fsd"
                    && parts[1].strip_suffix(".md").is_some_and(valid_slug)
            },
            _ => path == format!("{}.md", self.name()),
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validation {
    pub artifact_hash: String,
    pub dependency_hash: String,
    pub passed: bool,
    pub report: String,
    pub rubric_commit: String,
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
    Grooming,
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
    pub evidence_hash: String,
    pub rubric_commit: String,
    pub artifacts: BTreeMap<Phase, Artifact>,
    pub grooming_review: Option<Review>,
    pub publication_review: Option<Review>,
    pub publications: BTreeMap<String, Publication>,
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
