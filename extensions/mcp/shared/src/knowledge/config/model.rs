//! Model-provider configuration. `provider` selects the implementation of
//! the knowledge-bank crate's `model::ModelProvider`; the provider-specific
//! block beside it carries that provider's model ids and limits. `local` is the
//! shipped default: a free, deterministic, in-process provider that needs no
//! credential, so the server runs everywhere; `bedrock` is the swap-in for
//! production-grade embeddings and needs `bedrock_api_key`.

use serde::{Deserialize, Serialize};

/// Which model provider backs embeddings, chat, and reranking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    #[default]
    Local,
    Bedrock,
}

impl ProviderKind {
    #[must_use]
    pub const fn requires_api_key(self) -> bool {
        matches!(self, Self::Bedrock)
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Bedrock => "bedrock",
        }
    }
}

/// Provider selection plus the per-provider settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelConfig {
    #[serde(default)]
    pub provider: ProviderKind,
    #[serde(default)]
    pub local: LocalConfig,
    #[serde(default)]
    pub bedrock: BedrockConfig,
}

impl ModelConfig {
    // Why: the vector column is `vector(1024)`; whichever provider is active
    // must produce that width or every insert fails after the model call.
    #[must_use]
    pub const fn active_embed_dim(&self) -> u16 {
        match self.provider {
            ProviderKind::Local => self.local.embed_dim,
            ProviderKind::Bedrock => self.bedrock.embed_dim,
        }
    }
}

/// The in-process provider: feature-hashed n-gram embeddings and a BM25
/// reranker, no chat model, no network, no credential.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalConfig {
    #[serde(default = "default_local_embed_model")]
    pub embed_model: String,
    #[serde(default = "default_embed_dim")]
    pub embed_dim: u16,
    #[serde(default = "default_ngram")]
    pub ngram: u8,
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            embed_model: default_local_embed_model(),
            embed_dim: default_embed_dim(),
            ngram: default_ngram(),
        }
    }
}

fn default_local_embed_model() -> String {
    "local.hashed-ngram-v1".to_owned()
}
const fn default_ngram() -> u8 {
    2
}

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
