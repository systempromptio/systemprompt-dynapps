//! The model-provider seam (FR-MODEL-4 / NFR-PROV-1).
//!
//! Everything that talks to a model — embeddings for chunks and queries,
//! chat for chunk-boundary detection, enrichment and dating, and reranking —
//! goes through [`ModelProvider`]. Retrieval and ingestion hold the trait
//! object and never name a provider, so swapping one is a configuration
//! change plus one implementation. Two ship: [`LocalProvider`], the
//! credential-free default (hashed n-gram embeddings, BM25 rerank, no chat),
//! and [`BedrockProvider`], the Node system's original, selected by
//! `model.provider: bedrock` plus the `bedrock_api_key` secret.

pub mod bedrock;
pub mod local;

pub use bedrock::{BedrockProvider, assert_eu_region};
pub use local::LocalProvider;

use crate::config::{ModelConfig, ProviderKind};
use crate::error::KnowledgeBankError;
use async_trait::async_trait;
use std::sync::Arc;

/// A reranked document: its index in the input slice, and its relevance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RerankResult {
    pub index: usize,
    pub relevance_score: f64,
}

/// The capability boundary every model call crosses.
#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, KnowledgeBankError>;

    async fn converse(&self, system: &str, user: &str) -> Result<String, KnowledgeBankError>;

    async fn rerank(
        &self,
        query: &str,
        documents: &[String],
        top_n: usize,
    ) -> Result<Vec<RerankResult>, KnowledgeBankError>;

    fn embed_model(&self) -> &str;

    fn embed_dim(&self) -> usize;

    // Why: a provider without a chat model must not be asked for chunk
    // boundaries, enrichment or dates on every document — the pipeline reads
    // this once and takes the deterministic path silently instead of
    // logging a warning per call.
    fn supports_chat(&self) -> bool {
        true
    }
}

pub fn build_provider(
    config: &ModelConfig,
    api_key: Option<String>,
) -> Result<Arc<dyn ModelProvider>, KnowledgeBankError> {
    match config.provider {
        ProviderKind::Local => Ok(Arc::new(LocalProvider::new(config.local.clone()))),
        ProviderKind::Bedrock => {
            let api_key = api_key.ok_or_else(|| {
                KnowledgeBankError::Config(format!(
                    "model.provider is bedrock but the profile secret store has no \
                     `{}`; set it, or use model.provider: local (see \
                     docs/install/knowledge-db.md)",
                    crate::config::SECRET_KEY_NAME
                ))
            })?;
            Ok(Arc::new(BedrockProvider::new(
                config.bedrock.clone(),
                api_key,
            )?))
        },
    }
}
