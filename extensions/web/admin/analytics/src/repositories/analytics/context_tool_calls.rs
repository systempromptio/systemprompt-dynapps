//! Tool calls of a context's transcript requests, joined to the artifact the
//! ingest stored for each through the tool-call ledger.
//!
//! Split from `context_detail` so each file stays readable; the caller names
//! the request ids, exactly as it does for messages.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{AiRequestId, AiToolCallId, ArtifactId};

#[derive(Debug, Clone)]
pub struct ContextToolCallRow {
    pub request_id: AiRequestId,
    pub tool_name: String,
    pub sequence_number: i32,
    pub ai_tool_call_id: Option<AiToolCallId>,
    // JSON: variable-shape MCP tool arguments
    pub tool_input: serde_json::Value,
    // JSON: variable-shape MCP tool result
    pub tool_result_payload: Option<serde_json::Value>,
    pub tool_input_raw: Option<String>,
    pub tool_input_parse_error: Option<String>,
    pub artifact_id: Option<ArtifactId>,
    pub artifact_structured: bool,
    pub created_at: DateTime<Utc>,
}

pub async fn list_tool_calls_for_requests(
    pool: &PgPool,
    request_ids: &[String],
) -> Result<Vec<ContextToolCallRow>, sqlx::Error> {
    if request_ids.is_empty() {
        return Ok(Vec::new());
    }
    let rows = sqlx::query!(
        r#"
        SELECT
            t.request_id          AS "request_id!: AiRequestId",
            t.tool_name           AS "tool_name!",
            t.sequence_number     AS "sequence_number!",
            t.ai_tool_call_id     AS "ai_tool_call_id?: AiToolCallId",
            t.tool_input          AS "tool_input!",
            t.tool_result_payload AS "tool_result_payload?: serde_json::Value",
            l.artifact_id         AS "artifact_id: ArtifactId",
            COALESCE(l.is_structured, FALSE) AS "artifact_structured!",
            r.created_at          AS "created_at!"
        FROM ai_request_tool_calls t
        JOIN ai_requests r ON r.id = t.request_id
        LEFT JOIN tool_call_ledger l ON l.intent_id = t.id
        WHERE t.request_id = ANY($1)
        ORDER BY r.created_at ASC, t.sequence_number ASC
        "#,
        request_ids
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| {
            let (tool_input, tool_input_raw, tool_input_parse_error) =
                parse_tool_input(&r.tool_input);
            if tool_input_parse_error.is_some() {
                tracing::warn!(request_id = %r.request_id, sequence_number = r.sequence_number,
                "Stored tool arguments could not be parsed");
            }
            ContextToolCallRow {
                request_id: r.request_id,
                tool_name: r.tool_name,
                sequence_number: r.sequence_number,
                ai_tool_call_id: r.ai_tool_call_id,
                tool_input,
                tool_input_raw,
                tool_input_parse_error,
                tool_result_payload: r.tool_result_payload,
                artifact_id: r.artifact_id,
                artifact_structured: r.artifact_structured,
                created_at: r.created_at,
            }
        })
        .collect())
}

// JSON: Persisted MCP arguments may contain malformed legacy tool payloads.
#[must_use]
pub fn parse_tool_input(raw: &str) -> (serde_json::Value, Option<String>, Option<String>) {
    match serde_json::from_str(raw) {
        Ok(value) => (value, None, None),
        Err(error) => (
            serde_json::Value::Null,
            Some(raw.to_owned()),
            Some(format!(
                "Stored arguments could not be parsed ({:?}, line {}, column {}).",
                error.classify(),
                error.line(),
                error.column()
            )),
        ),
    }
}
