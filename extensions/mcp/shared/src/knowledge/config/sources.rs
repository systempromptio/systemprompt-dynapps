//! The source catalogue and the ingestion-side settings.
//!
//! A source category is a `kind` (where documents come from), a chunking
//! `strategy` (how they are split), and the scope that kind reads — spaces or
//! projects for Atlassian, a directory tree for files. Who may read a
//! document depends on the kind: an Atlassian document is granted to every
//! caller who synced it, because Atlassian already enforced the read for
//! them; a file-sourced document is hand-authored team material and is
//! granted to the subjects the category configures, `role:user` (everyone
//! signed in) unless it says otherwise. Adding a category is a configuration
//! change (FR-ING-8): every kind-specific field lives on this one struct and
//! is checked by `validate`, not by code that knows the category names.

use crate::knowledge::principal::Subject;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Where a source's documents are read from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    Files,
    #[serde(rename = "google_drive")]
    GoogleDrive,
    Confluence,
    Jira,
}

impl SourceKind {
    #[must_use]
    pub const fn is_atlassian(self) -> bool {
        matches!(self, Self::Confluence | Self::Jira)
    }
}

/// Per-source chunking strategy, carried over from the Node system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChunkStrategy {
    Llm,
    Script,
    None,
}

/// A subject every document in a files category is granted to.
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
            kind: crate::knowledge::principal::SUBJECT_ROLE.to_owned(),
            value: "user".to_owned(),
        }
    }

    #[must_use]
    pub fn subject(&self) -> Subject {
        Subject {
            kind: self.kind.clone(),
            value: self.value.clone(),
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
    pub grants: Vec<GrantConfig>,
    #[serde(default)]
    pub spaces: Vec<String>,
    #[serde(default)]
    pub projects: Vec<String>,
    #[serde(default)]
    pub jql: Option<String>,
}

impl SourceConfig {
    // Why: an empty `grants:` means "everyone signed in", never "nobody" —
    // fail-closed is for content whose readers are unknown, not for content
    // the operator simply did not annotate.
    #[must_use]
    pub fn default_grants(&self) -> Vec<GrantConfig> {
        if self.grants.is_empty() {
            vec![GrantConfig::everyone_signed_in()]
        } else {
            self.grants.clone()
        }
    }

    #[must_use]
    pub fn configured_subjects(&self) -> Vec<Subject> {
        self.default_grants()
            .iter()
            .map(GrantConfig::subject)
            .collect()
    }

    // Why: a files category reads its own `path` when set, otherwise
    // `<files.root>/<id>`.
    #[must_use]
    pub fn resolved_path(&self, id: &str, files: &FilesSettings) -> PathBuf {
        self.path
            .as_deref()
            .map_or_else(|| PathBuf::from(&files.root).join(id), PathBuf::from)
    }
}

/// Where file-sourced categories live by default.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FilesSettings {
    #[serde(default = "default_files_root")]
    pub root: String,
}

impl Default for FilesSettings {
    fn default() -> Self {
        Self {
            root: default_files_root(),
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

/// Ingestion concurrency. There is no schedule: a sync runs when a user asks
/// for one.
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

/// Atlassian-side settings; no credential lives here.
///
/// Reads go through each caller's connected Atlassian MCP. `base_url` is the
/// site every citation points at; `cloud_id` picks the site when a connected
/// account reaches more than one.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AtlassianSettings {
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub cloud_id: Option<String>,
    #[serde(default = "default_true")]
    pub include_comments: bool,
}

impl Default for AtlassianSettings {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            cloud_id: None,
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
fn default_files_root() -> String {
    "storage/knowledge".to_owned()
}
