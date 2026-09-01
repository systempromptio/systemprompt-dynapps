//! The model-provider seam (FR-MODEL-4 / NFR-PROV-1).
//!
//! Everything that talks to a model — embeddings for chunks and queries,
//! chat for chunk-boundary detection, enrichment and dating, and reranking —
//! goes through [`ModelProvider`]. Retrieval and ingestion hold the trait
//! object and never name a provider, so swapping one is a configuration
//! change plus one implementation. Bedrock is the only implementation today,
//! as it was in the Node system this crate migrates.

pub mod bedrock;

pub use bedrock::{BedrockProvider, assert_eu_region};

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
}

pub fn build_provider(
    config: &ModelConfig,
    api_key: String,
) -> Result<Arc<dyn ModelProvider>, KnowledgeBankError> {
    match config.provider {
        ProviderKind::Bedrock => Ok(Arc::new(BedrockProvider::new(
            config.bedrock.clone(),
            api_key,
        )?)),
    }
}
