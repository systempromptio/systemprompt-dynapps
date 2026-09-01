//! A deterministic `ModelProvider` for the database-backed tests: embeddings
//! derived from the text so identical text embeds identically, a rerank that
//! can be switched to fail, a chat that never gives a date, and counters that
//! let a test prove "no model call happened".

use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use systemprompt_mcp_knowledge_bank::error::KnowledgeBankError;
use systemprompt_mcp_knowledge_bank::{ModelProvider, RerankResult};

#[derive(Default)]
pub struct FakeModel {
    pub embed_calls: AtomicUsize,
    pub converse_calls: AtomicUsize,
    pub rerank_calls: AtomicUsize,
    pub fail_rerank: AtomicBool,
}

impl FakeModel {
    pub fn embeds(&self) -> usize {
        self.embed_calls.load(Ordering::SeqCst)
    }
}

// Deterministic, non-degenerate: a handful of dimensions are set from byte
// positions so different texts land at different points of the sphere.
fn embedding_for(text: &str) -> Vec<f32> {
    let mut vector = vec![0.0f32; 1024];
    for (i, byte) in text.bytes().enumerate() {
        let slot = (usize::from(byte) * 31 + i) % 1024;
        vector[slot] += 1.0;
    }
    let norm = vector.iter().map(|v| v * v).sum::<f32>().sqrt().max(1.0);
    vector.iter_mut().for_each(|v| *v /= norm);
    vector
}

#[async_trait]
impl ModelProvider for FakeModel {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, KnowledgeBankError> {
        self.embed_calls.fetch_add(1, Ordering::SeqCst);
        Ok(embedding_for(text))
    }

    async fn converse(&self, _system: &str, _user: &str) -> Result<String, KnowledgeBankError> {
        self.converse_calls.fetch_add(1, Ordering::SeqCst);
        Ok("NONE".to_owned())
    }

    async fn rerank(
        &self,
        _query: &str,
        documents: &[String],
        top_n: usize,
    ) -> Result<Vec<RerankResult>, KnowledgeBankError> {
        self.rerank_calls.fetch_add(1, Ordering::SeqCst);
        if self.fail_rerank.load(Ordering::SeqCst) {
            return Err(KnowledgeBankError::Backend("rerank unavailable".to_owned()));
        }
        Ok((0..documents.len().min(top_n))
            .map(|index| RerankResult {
                index,
                relevance_score: 1.0 - index as f64 * 0.05,
            })
            .collect())
    }

    fn embed_model(&self) -> &str {
        "fake-embed-v1"
    }

    fn embed_dim(&self) -> usize {
        1024
    }
}
