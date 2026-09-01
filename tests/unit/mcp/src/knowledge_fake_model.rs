//! A scripted `ModelProvider` for the pure tests: canned chat replies, a
//! deterministic embedding, and a rerank that preserves input order.

use async_trait::async_trait;
use std::sync::Mutex;
use systemprompt_mcp_knowledge_bank::error::KnowledgeBankError;
use systemprompt_mcp_knowledge_bank::{ModelProvider, RerankResult};

pub struct FakeModel {
    replies: Mutex<Vec<String>>,
    pub converse_calls: Mutex<usize>,
}

impl FakeModel {
    pub fn replying(replies: &[&str]) -> Self {
        Self {
            replies: Mutex::new(replies.iter().rev().map(|r| (*r).to_owned()).collect()),
            converse_calls: Mutex::new(0),
        }
    }
}

#[async_trait]
impl ModelProvider for FakeModel {
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, KnowledgeBankError> {
        Ok(vec![0.0; 1024])
    }

    async fn converse(&self, _system: &str, _user: &str) -> Result<String, KnowledgeBankError> {
        *self.converse_calls.lock().expect("lock") += 1;
        self.replies
            .lock()
            .expect("lock")
            .pop()
            .ok_or_else(|| KnowledgeBankError::Backend("no scripted reply".to_owned()))
    }

    async fn rerank(
        &self,
        _query: &str,
        documents: &[String],
        top_n: usize,
    ) -> Result<Vec<RerankResult>, KnowledgeBankError> {
        Ok((0..documents.len().min(top_n))
            .map(|index| RerankResult {
                index,
                relevance_score: 1.0 - index as f64 * 0.1,
            })
            .collect())
    }

    fn embed_model(&self) -> &str {
        "fake-embed"
    }

    fn embed_dim(&self) -> usize {
        1024
    }
}
