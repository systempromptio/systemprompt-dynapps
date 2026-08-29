//! Cohere rerank over `bedrock-runtime`.
//!
//! The call goes through `InvokeModel`, not the `bedrock-agent-runtime`
//! `Rerank` operation: that operation rejects bearer-token auth, and this
//! instance's key has no `bedrock-agent` access at all (probed: 403). The
//! Node system this crate ports carries the same note.

use super::BedrockClient;
use crate::error::KnowledgeBankError;
use serde_json::json;

/// A reranked document: its index in the input slice, and its relevance.
#[derive(Debug, Clone, Copy)]
pub struct RerankResult {
    pub index: usize,
    pub relevance_score: f64,
}

pub async fn rerank(
    client: &BedrockClient,
    query: &str,
    documents: &[String],
    top_n: usize,
) -> Result<Vec<RerankResult>, KnowledgeBankError> {
    if documents.is_empty() {
        return Ok(Vec::new());
    }
    let body = json!({
        "api_version": 2,
        "query": query,
        "documents": documents,
        "top_n": top_n.min(documents.len()),
    });
    let response = client
        .invoke_model(&client.config.rerank_model, &body)
        .await?;

    // JSON: protocol boundary. Cohere's rerank response is parsed straight
    // into RerankResult below; the intermediate Value never escapes this
    // function.
    let results = response
        .get("results")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            KnowledgeBankError::Internal("cohere response carried no `results` array".to_owned())
        })?;

    results
        .iter()
        .map(|entry| {
            let index = entry
                .get("index")
                .and_then(serde_json::Value::as_u64)
                .ok_or_else(|| {
                    KnowledgeBankError::Internal("rerank result missing `index`".to_owned())
                })?;
            let score = entry
                .get("relevance_score")
                .and_then(serde_json::Value::as_f64)
                .ok_or_else(|| {
                    KnowledgeBankError::Internal(
                        "rerank result missing `relevance_score`".to_owned(),
                    )
                })?;
            // Why: TryFromIntError is variant-less.
            // lint-ok: error-adapt
            let index = usize::try_from(index).map_err(|e| {
                KnowledgeBankError::Internal(format!("rerank index out of range: {e}"))
            })?;
            if index >= documents.len() {
                return Err(KnowledgeBankError::Internal(format!(
                    "rerank index {index} exceeds {} documents",
                    documents.len()
                )));
            }
            Ok(RerankResult {
                index,
                relevance_score: score,
            })
        })
        .collect()
}
