//! The local model provider: free, deterministic, in-process, offline.
//!
//! Embeddings are feature-hashed word n-grams ([`embed`]) and reranking is
//! BM25 over the candidate set ([`rerank`]); neither touches the network or
//! needs a credential, so the knowledge bank runs with real tools over real
//! pgvector on every machine, CI included. There is no chat model:
//! [`ModelProvider::supports_chat`] is `false`, `converse` is an error, and
//! the ingestion pipeline takes the same deterministic fallbacks it takes
//! when a remote chat model fails — script chunking, no enrichment,
//! frontmatter-or-mtime dating. Swapping to Bedrock is `model.provider:
//! bedrock` plus the `bedrock_api_key` secret, followed by
//! `sync_knowledge {rebuild: true}` because the embedding space changes.

mod embed;
mod rerank;

use super::{ModelProvider, RerankResult};
use crate::config::LocalConfig;
use crate::error::KnowledgeBankError;
use async_trait::async_trait;

/// The in-process provider built from [`LocalConfig`].
#[derive(Debug, Clone)]
pub struct LocalProvider {
    config: LocalConfig,
}

impl LocalProvider {
    #[must_use]
    pub const fn new(config: LocalConfig) -> Self {
        Self { config }
    }

    #[must_use]
    pub const fn config(&self) -> &LocalConfig {
        &self.config
    }
}

#[async_trait]
impl ModelProvider for LocalProvider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, KnowledgeBankError> {
        Ok(embed::embed(
            text,
            usize::from(self.config.embed_dim),
            usize::from(self.config.ngram),
        ))
    }

    async fn converse(&self, _system: &str, _user: &str) -> Result<String, KnowledgeBankError> {
        Err(KnowledgeBankError::NotConfigured(
            "the local model provider has no chat model; set model.provider: bedrock for \
             LLM chunking, enrichment and dating"
                .to_owned(),
        ))
    }

    async fn rerank(
        &self,
        query: &str,
        documents: &[String],
        top_n: usize,
    ) -> Result<Vec<RerankResult>, KnowledgeBankError> {
        Ok(rerank::rerank(query, documents, top_n))
    }

    fn embed_model(&self) -> &str {
        &self.config.embed_model
    }

    fn embed_dim(&self) -> usize {
        usize::from(self.config.embed_dim)
    }

    fn supports_chat(&self) -> bool {
        false
    }
}
