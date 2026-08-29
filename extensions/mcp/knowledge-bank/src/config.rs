//! Configuration for the knowledge bank: Bedrock models, chunking targets,
//! retrieval tunables, and the source catalogue.
//!
//! Values mirror the Node "project-context" `config.yaml` this crate is a
//! migration of, so a ported index behaves identically. The file is read
//! directly by this binary from `services/knowledge-bank/config.yaml` (path
//! overridable with `KNOWLEDGE_BANK_CONFIG`); it is deliberately NOT part of
//! the `services/config/config.yaml` aggregator, whose `includes:` list only
//! carries resources the core `ServicesConfig` knows how to type.
//!
//! The Bedrock API key is never read from here — it comes from the profile
//! secret store under `bedrock_api_key`, so no credential can reach a tracked
//! file. See `docs/tickets/req-047-knowledge-bank-mcp.md`.

use crate::error::KnowledgeBankError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const DEFAULT_CONFIG_PATH: &str = "services/knowledge-bank/config.yaml";
pub const CONFIG_PATH_ENV: &str = "KNOWLEDGE_BANK_CONFIG";
pub const SECRET_KEY_NAME: &str = "bedrock_api_key";

/// Bedrock model ids, region, and concurrency ceilings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BedrockConfig {
    #[serde(default = "default_region")]
    pub region: String,
    #[serde(default = "default_embed_model")]
    pub embed_model: String,
    #[serde(default = "default_embed_dim")]
    pub embed_dim: u16,
    #[serde(default = "default_chunk_model")]
    pub chunk_model: String,
    #[serde(default = "default_rerank_model")]
    pub rerank_model: String,
    #[serde(default = "default_embed_concurrency")]
    pub embed_concurrency: usize,
    #[serde(default = "default_max_concurrency")]
    pub max_concurrency: usize,
}

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

/// One knowledge source category and how its documents are chunked.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceConfig {
    pub strategy: ChunkStrategy,
    pub description: String,
    #[serde(default)]
    pub archived: bool,
}

/// Per-source chunking strategy, carried over from the Node system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChunkStrategy {
    Llm,
    Script,
    None,
}

/// The whole knowledge-bank configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeBankConfig {
    #[serde(default)]
    pub bedrock: BedrockConfig,
    #[serde(default)]
    pub chunking: ChunkingConfig,
    #[serde(default)]
    pub retrieval: RetrievalConfig,
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
            .map_err(|e| KnowledgeBankError::Internal(format!("read {}: {e}", path.display())))?;
        serde_yaml::from_str(&raw)
            // Why: same adaptation as the read above.
            // lint-ok: error-adapt
            .map_err(|e| KnowledgeBankError::Internal(format!("parse {}: {e}", path.display())))
    }

    pub fn live_source_ids(&self) -> Vec<String> {
        self.sources
            .iter()
            .filter(|(_, cfg)| !cfg.archived)
            .map(|(id, _)| id.clone())
            .collect()
    }
}

impl Default for BedrockConfig {
    fn default() -> Self {
        Self {
            region: default_region(),
            embed_model: default_embed_model(),
            embed_dim: default_embed_dim(),
            chunk_model: default_chunk_model(),
            rerank_model: default_rerank_model(),
            embed_concurrency: default_embed_concurrency(),
            max_concurrency: default_max_concurrency(),
        }
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

fn default_region() -> String {
    "eu-central-1".to_owned()
}
fn default_embed_model() -> String {
    "amazon.titan-embed-text-v2:0".to_owned()
}
const fn default_embed_dim() -> u16 {
    1024
}
fn default_chunk_model() -> String {
    "openai.gpt-oss-120b-1:0".to_owned()
}
fn default_rerank_model() -> String {
    "cohere.rerank-v3-5:0".to_owned()
}
const fn default_embed_concurrency() -> usize {
    16
}
const fn default_max_concurrency() -> usize {
    8
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
