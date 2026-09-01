//! Single-turn chat through the Bedrock Converse API — the generic
//! `llm(system, user)` that chunk-boundary detection, enrichment, and dating
//! drive. Temperature is pinned to zero so the ingestion-side decisions are as
//! stable as the model allows; the callers cache the results anyway.

use super::BedrockProvider;
use crate::error::KnowledgeBankError;
use serde_json::json;

const MAX_TOKENS: u32 = 4096;

pub(super) async fn converse(
    provider: &BedrockProvider,
    system: &str,
    user: &str,
) -> Result<String, KnowledgeBankError> {
    let body = json!({
        "system": [{"text": system}],
        "messages": [{"role": "user", "content": [{"text": user}]}],
        "inferenceConfig": {"temperature": 0, "maxTokens": MAX_TOKENS},
    });
    let response = provider
        .converse_model(&provider.config.chunk_model, &body)
        .await?;

    // JSON: protocol boundary. The Converse response nests the assistant
    // turn under output.message.content[]; only the text parts are wanted.
    let parts = response
        .pointer("/output/message/content")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            KnowledgeBankError::Backend("converse response carried no message content".to_owned())
        })?;
    let text = parts
        .iter()
        .filter_map(|part| part.get("text").and_then(serde_json::Value::as_str))
        .collect::<Vec<_>>()
        .join("");
    Ok(text.trim().to_owned())
}
