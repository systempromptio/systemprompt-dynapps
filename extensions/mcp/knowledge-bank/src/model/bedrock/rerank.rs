//! Cohere rerank over `bedrock-runtime`.
//!
//! The call goes through `InvokeModel`, not the `bedrock-agent-runtime`
//! `Rerank` operation: that operation rejects bearer-token auth, and this
//! instance's key has no `bedrock-agent` access at all (probed: 403). The
//! Node system this crate ports carries the same note.

use super::BedrockProvider;
use crate::error::KnowledgeBankError;
use crate::model::RerankResult;
use serde_json::json;

pub(super) async fn rerank(
    provider: &BedrockProvider,
    query: &str,
    documents: &[String],
    top_n: usize,
) -> Result<Vec<RerankResult>, KnowledgeBankError> {
    if documents.is_empty() {
        return Ok(Vec::new());
    }
    let mut body = json!({
        "query": query,
        "documents": documents,
        "top_n": top_n.min(documents.len()),
    });
    // Why: the Cohere family wants an explicit API version; Amazon's rerank
    // models reject the field. Same branch the Node connector carries.
    if provider.config.rerank_model.starts_with("cohere.") {
        body["api_version"] = json!(2);
    }
    let response = provider
        .invoke_model(&provider.config.rerank_model, &body)
        .await?;

    // JSON: protocol boundary. Cohere's rerank response is parsed straight
    // into RerankResult below; the intermediate Value never escapes this
    // function.
    let results = response
        .get("results")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            KnowledgeBankError::Backend("rerank response carried no `results` array".to_owned())
        })?;

    results
        .iter()
        .map(|entry| {
            let index = entry
                .get("index")
                .and_then(serde_json::Value::as_u64)
                .ok_or_else(|| {
                    KnowledgeBankError::Backend("rerank result missing `index`".to_owned())
                })?;
            let score = entry
                .get("relevance_score")
                .and_then(serde_json::Value::as_f64)
                .ok_or_else(|| {
                    KnowledgeBankError::Backend(
                        "rerank result missing `relevance_score`".to_owned(),
                    )
                })?;
            // Why: TryFromIntError is variant-less.
            // lint-ok: error-adapt
            let index = usize::try_from(index).map_err(|e| {
                KnowledgeBankError::Backend(format!("rerank index out of range: {e}"))
            })?;
            if index >= documents.len() {
                return Err(KnowledgeBankError::Backend(format!(
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
