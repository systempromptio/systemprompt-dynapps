//! Titan v2 text embeddings.
//!
//! One text per `InvokeModel` call — the Titan embedding API takes no batch
//! form — so callers fan out under the provider's embed semaphore rather than
//! building a batch request.

use super::BedrockProvider;
use crate::error::KnowledgeBankError;
use serde_json::json;

pub(super) async fn embed_text(
    provider: &BedrockProvider,
    text: &str,
) -> Result<Vec<f32>, KnowledgeBankError> {
    let _permit = provider
        .embeds
        .acquire()
        .await
        // Why: AcquireError is variant-less.
        // lint-ok: error-adapt
        .map_err(|e| KnowledgeBankError::Internal(format!("semaphore closed: {e}")))?;

    let body = json!({
        "inputText": text,
        "dimensions": provider.config.embed_dim,
        "normalize": true,
    });
    let response = provider
        .invoke_model(&provider.config.embed_model, &body)
        .await?;

    // JSON: protocol boundary. Titan's response is read positionally into a
    // Vec<f32> here rather than modelled, because the only field this crate
    // ever wants from it is `embedding`.
    let values = response
        .get("embedding")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            KnowledgeBankError::Backend("titan response carried no `embedding` array".to_owned())
        })?;

    let vector: Vec<f32> = values
        .iter()
        .filter_map(serde_json::Value::as_f64)
        .map(|v| v as f32)
        .collect();

    let expected = usize::from(provider.config.embed_dim);
    if vector.len() != expected {
        return Err(KnowledgeBankError::Backend(format!(
            "titan returned {} dimensions, expected {expected}",
            vector.len()
        )));
    }
    Ok(vector)
}
