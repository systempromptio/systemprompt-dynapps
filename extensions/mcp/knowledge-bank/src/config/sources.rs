//! The source catalogue and the ingestion-side settings.
//!
//! A source category is a `kind` (which Atlassian product documents come
//! from), a chunking `strategy` (how they are split), and the scope the
//! caller's own credentials are pointed at — spaces or projects. Who may read
//! a document is not configured here at all: every document a caller syncs is
//! granted to that caller, because Atlassian already enforced the read on
//! their behalf. Adding a category is a configuration change (FR-ING-8):
//! every kind-specific field lives on this one struct and is checked by
//! `validate`, not by code that knows the category names.

use serde::{Deserialize, Serialize};

/// Where a source's documents are read from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
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
    pub spaces: Vec<String>,
    #[serde(default)]
    pub projects: Vec<String>,
    #[serde(default)]
    pub jql: Option<String>,
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

/// Ingestion concurrency. There is no schedule: a sync runs when a user asks
/// for one with their own credentials.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IngestionConfig {
    #[serde(default = "default_doc_concurrency")]
    pub doc_concurrency: usize,
}

impl Default for IngestionConfig {
    fn default() -> Self {
        Self {
            doc_concurrency: default_doc_concurrency(),
        }
    }
}

/// Where the index lives.
///
/// The name of the profile secret holding the knowledge database URL. When
/// the secret is absent the instance pool is used, which is what local
/// development and compose deployments want.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeDatabaseConfig {
    #[serde(default = "default_knowledge_secret")]
    pub secret: String,
}

impl Default for KnowledgeDatabaseConfig {
    fn default() -> Self {
        Self {
            secret: default_knowledge_secret(),
        }
    }
}

/// Atlassian-side settings that are not credentials. Credentials arrive on
/// each `sync_knowledge` call as the caller's own headers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AtlassianSettings {
    #[serde(default)]
    pub base_url: String,
    #[serde(default = "default_true")]
    pub include_comments: bool,
}

impl Default for AtlassianSettings {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            include_comments: true,
        }
    }
}

const fn default_true() -> bool {
    true
}
const fn default_doc_concurrency() -> usize {
    6
}
fn default_knowledge_secret() -> String {
    "knowledge_database_url".to_owned()
}
