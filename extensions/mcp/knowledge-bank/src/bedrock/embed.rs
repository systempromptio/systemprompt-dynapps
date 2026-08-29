//! Titan v2 text embeddings.
//!
//! One text per `InvokeModel` call — the Titan embedding API takes no batch
//! form — so callers fan out under the client's embed semaphore rather than
//! building a batch request.

use super::BedrockClient;
use crate::error::KnowledgeBankError;
use serde_json::json;

pub async fn embed_text(
    client: &BedrockClient,
    text: &str,
) -> Result<Vec<f32>, KnowledgeBankError> {
    let _permit = client
        .embed_permits()
        .acquire()
        .await
        // Why: AcquireError is variant-less.
        // lint-ok: error-adapt
        .map_err(|e| KnowledgeBankError::Internal(format!("semaphore closed: {e}")))?;

    let body = json!({
        "inputText": text,
        "dimensions": client.config.embed_dim,
        "normalize": true,
    });
    let response = client
        .invoke_model(&client.config.embed_model, &body)
        .await?;

    // JSON: protocol boundary. Titan's response is read positionally into a
    // Vec<f32> here rather than modelled, because the only field this crate
    // ever wants from it is `embedding`.
    let values = response
        .get("embedding")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            KnowledgeBankError::Internal("titan response carried no `embedding` array".to_owned())
        })?;

    let vector: Vec<f32> = values
        .iter()
        .filter_map(serde_json::Value::as_f64)
        .map(|v| v as f32)
        .collect();

    let expected = usize::from(client.config.embed_dim);
    if vector.len() != expected {
        return Err(KnowledgeBankError::Internal(format!(
            "titan returned {} dimensions, expected {expected}",
            vector.len()
        )));
    }
    Ok(vector)
}
