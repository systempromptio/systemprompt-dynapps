//! Configuration for the knowledge bank: model provider, chunking targets,
//! retrieval tunables, ingestion concurrency, where the index lives, and the
//! source catalogue.
//!
//! Values mirror the Node "project-context" `config.yaml` this crate is a
//! migration of, so a ported corpus behaves identically. The file is read
//! directly by this binary from `services/knowledge-bank/config.yaml` (path
//! overridable with `KNOWLEDGE_BANK_CONFIG`); it is deliberately NOT part of
//! the `services/config/config.yaml` aggregator, whose `includes:` list only
//! carries resources the core `ServicesConfig` knows how to type.
//!
//! No credential is read from here. The Bedrock key (only when
//! `model.provider: bedrock`) and the optional knowledge-database URL come
//! from the profile secret store; Atlassian is read through each caller's
//! connected account via this instance's own MCP gateway
//! (see the knowledge-bank crate's `ingest::sources::atlassian`). File-sourced
//! categories need no credential at all: they read a directory under
//! `files.root`.

mod model;
mod sources;
mod validate;

pub use model::{BedrockConfig, LocalConfig, ModelConfig, ProviderKind};
pub use sources::{
    AtlassianSettings, ChunkStrategy, DatesConfig, FilesSettings, GrantConfig, IngestionConfig,
    KnowledgeDatabaseConfig, SourceConfig, SourceKind,
};

use crate::knowledge::error::KnowledgeBankError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const DEFAULT_CONFIG_PATH: &str = "services/knowledge-bank/config.yaml";
pub const CONFIG_PATH_ENV: &str = "KNOWLEDGE_BANK_CONFIG";
pub const FILES_ROOT_ENV: &str = "KNOWLEDGE_BANK_FILES_ROOT";
pub const SECRET_KEY_NAME: &str = "bedrock_api_key";

/// Chunk-size targets for the ingestion splitter, in characters.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChunkingConfig {
    #[serde(default = "default_target_chars")]
    pub target_chars: usize,
    #[serde(default = "default_max_chars")]
    pub max_chars: usize,
    #[serde(default = "default_min_chars")]
    pub min_chars: usize,
    #[serde(default = "default_split_tolerance")]
    pub split_tolerance: usize,
    #[serde(default = "default_enrich")]
    pub enrich: bool,
}

impl ChunkingConfig {
    #[must_use]
    pub const fn ceiling(&self) -> usize {
        self.max_chars + self.split_tolerance
    }
}

/// Serving-path tunables for hybrid search.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetrievalConfig {
    #[serde(default = "default_top_k")]
    pub top_k: u8,
    #[serde(default = "default_candidate_multiplier")]
    pub candidate_multiplier: u8,
    #[serde(default = "default_rerank_enabled")]
    pub rerank_enabled: bool,
}

/// The whole knowledge-bank configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeBankConfig {
    #[serde(default)]
    pub google_drive: crate::knowledge::drive::config::DriveSettings,
    #[serde(default)]
    pub model: ModelConfig,
    #[serde(default)]
    pub chunking: ChunkingConfig,
    #[serde(default)]
    pub retrieval: RetrievalConfig,
    #[serde(default)]
    pub dates: DatesConfig,
    #[serde(default)]
    pub ingestion: IngestionConfig,
    #[serde(default)]
    pub knowledge_database: KnowledgeDatabaseConfig,
    #[serde(default)]
    pub atlassian: AtlassianSettings,
    #[serde(default)]
    pub files: FilesSettings,
    #[serde(default)]
    pub sources: BTreeMap<String, SourceConfig>,
}

impl KnowledgeBankConfig {
    pub fn resolve_path() -> PathBuf {
        std::env::var(CONFIG_PATH_ENV)
            .map_or_else(|_| PathBuf::from(DEFAULT_CONFIG_PATH), PathBuf::from)
    }

    pub fn load(path: &Path) -> Result<Self, KnowledgeBankError> {
        if !path.is_file() {
            tracing::warn!(path = %path.display(), "no knowledge-bank config, using defaults");
            return Ok(Self::default());
        }
        // Why: io::Error and serde_yaml::Error carry no variant worth
        // preserving here, and neither names the file it failed on — which is
        // the only detail an operator needs in order to act.
        // lint-ok: error-adapt
        let raw = std::fs::read_to_string(path)
            .map_err(|e| KnowledgeBankError::Config(format!("read {}: {e}", path.display())))?;
        let config = Self::parse(&raw).map_err(|e| match e {
            // Why: same adaptation as the read above — the parse error has
            // to name the file.
            // lint-ok: error-adapt
            KnowledgeBankError::Config(msg) => {
                KnowledgeBankError::Config(format!("{}: {msg}", path.display()))
            },
            other => other,
        })?;
        Ok(config.with_files_root_override(std::env::var(FILES_ROOT_ENV).ok()))
    }

    // Why: the YAML is baked into the image, but where a deployment keeps
    // its persistent disk is the host's business — on Fly a volume mount,
    // in compose a named volume. The env override points `files.root` there
    // without a per-target config file.
    #[must_use]
    pub fn with_files_root_override(mut self, root: Option<String>) -> Self {
        if let Some(root) = root.map(|r| r.trim().to_owned()).filter(|r| !r.is_empty()) {
            self.files.root = root;
        }
        self
    }

    pub fn parse(raw: &str) -> Result<Self, KnowledgeBankError> {
        // Why: serde_yaml::Error is variant-less for our purposes; the text is
        // the whole diagnostic.
        // lint-ok: error-adapt
        let config: Self =
            serde_yaml::from_str(raw).map_err(|e| KnowledgeBankError::Config(e.to_string()))?;
        validate::validate(&config)?;
        Ok(config)
    }

    #[must_use]
    // Why: a `google_drive` category is only live while the Drive feature is
    // switched on; with it off the category has no ingestion path.
    pub fn live_source_ids(&self) -> Vec<String> {
        self.sources
            .iter()
            .filter(|(_, cfg)| !cfg.archived)
            .filter(|(_, cfg)| cfg.kind != SourceKind::GoogleDrive || self.google_drive.enabled)
            .map(|(id, _)| id.clone())
            .collect()
    }

    #[must_use]
    pub fn source(&self, id: &str) -> Option<&SourceConfig> {
        self.sources.get(id)
    }

    #[must_use]
    pub fn has_atlassian_sources(&self) -> bool {
        self.sources.values().any(|s| s.kind.is_atlassian())
    }

    // Why: a sync of `selected` (every source when empty) needs the caller's
    // connected Atlassian account only if an Atlassian category is in it.
    #[must_use]
    pub fn requires_atlassian(&self, selected: &[String]) -> bool {
        self.sources
            .iter()
            .filter(|(id, _)| selected.is_empty() || selected.contains(id))
            .any(|(_, s)| s.kind.is_atlassian())
    }

    // Why: the files categories are the ones a sync can run without a
    // connected Atlassian account.
    #[must_use]
    pub fn files_source_ids(&self) -> Vec<String> {
        self.sources
            .iter()
            .filter(|(_, s)| s.kind == SourceKind::Files)
            .map(|(id, _)| id.clone())
            .collect()
    }

    #[must_use]
    pub fn atlassian_source_ids(&self) -> Vec<String> {
        self.sources
            .iter()
            .filter(|(_, s)| s.kind.is_atlassian())
            .map(|(id, _)| id.clone())
            .collect()
    }
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            target_chars: default_target_chars(),
            max_chars: default_max_chars(),
            min_chars: default_min_chars(),
            split_tolerance: default_split_tolerance(),
            enrich: default_enrich(),
        }
    }
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            top_k: default_top_k(),
            candidate_multiplier: default_candidate_multiplier(),
            rerank_enabled: default_rerank_enabled(),
        }
    }
}

const fn default_target_chars() -> usize {
    1600
}
const fn default_max_chars() -> usize {
    3000
}
const fn default_min_chars() -> usize {
    300
}
const fn default_split_tolerance() -> usize {
    100
}
const fn default_enrich() -> bool {
    true
}
const fn default_top_k() -> u8 {
    5
}
const fn default_candidate_multiplier() -> u8 {
    4
}
const fn default_rerank_enabled() -> bool {
    true
}
