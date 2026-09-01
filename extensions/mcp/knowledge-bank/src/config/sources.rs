//! The source catalogue and the ingestion-side settings.
//!
//! A source category is a `kind` (where documents come from), a chunking
//! `strategy` (how they are split), and the `grants` every unrestricted
//! document in it receives. Adding a category is a configuration change —
//! FR-ING-8 — because every kind-specific field lives on this one struct and
//! is checked by `validate`, not by code that knows the category names.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Where a source's documents are read from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    Files,
    Confluence,
    Jira,
}

/// Per-source chunking strategy, carried over from the Node system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChunkStrategy {
    Llm,
    Script,
    None,
}

/// A subject every unrestricted document in a source is granted to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrantConfig {
    pub kind: String,
    pub value: String,
}

impl GrantConfig {
    #[must_use]
    pub fn everyone_signed_in() -> Self {
        Self {
            kind: "role".to_owned(),
            value: "user".to_owned(),
        }
    }
}

/// One knowledge source category.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceConfig {
    pub kind: SourceKind,
    pub strategy: ChunkStrategy,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub spaces: Vec<String>,
    #[serde(default)]
    pub projects: Vec<String>,
    #[serde(default)]
    pub jql: Option<String>,
    #[serde(default)]
    pub grants: Vec<GrantConfig>,
}

impl SourceConfig {
    // Why: an empty `grants:` means "everyone signed in", never "nobody" —
    // fail-closed is reserved for content whose restrictions could not be
    // read, not for content the operator simply did not annotate.
    #[must_use]
    pub fn default_grants(&self) -> Vec<GrantConfig> {
        if self.grants.is_empty() {
            vec![GrantConfig::everyone_signed_in()]
        } else {
            self.grants.clone()
        }
    }
}

/// Document dating: whether the LLM step runs between frontmatter and mtime.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatesConfig {
    #[serde(default = "default_true")]
    pub extract_via_llm: bool,
}

impl Default for DatesConfig {
    fn default() -> Self {
        Self {
            extract_via_llm: true,
        }
    }
}

/// The in-process ingestion schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IngestionConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_schedule")]
    pub schedule: String,
    #[serde(default = "default_doc_concurrency")]
    pub doc_concurrency: usize,
}

impl Default for IngestionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            schedule: default_schedule(),
            doc_concurrency: default_doc_concurrency(),
        }
    }
}

/// Atlassian-side settings that are not credentials.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AtlassianSettings {
    #[serde(default = "default_true")]
    pub include_comments: bool,
    #[serde(default)]
    pub group_map: BTreeMap<String, String>,
}

const fn default_true() -> bool {
    true
}
fn default_schedule() -> String {
    "0 0 * * * *".to_owned()
}
const fn default_doc_concurrency() -> usize {
    6
}
