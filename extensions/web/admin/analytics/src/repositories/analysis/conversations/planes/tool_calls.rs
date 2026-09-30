//! Tool calls of one conversation from `tool_activity` — intent, execution and
//! result folded into one row with the artifact rule's verdict, matched by
//! context or by the client session.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{ArtifactId, ContextId};

#[derive(Debug, Clone)]
pub struct ConversationToolCallRow {
    pub tool_name: Option<String>,
    pub server_name: Option<String>,
    pub state: String,
    pub source: Option<String>,
    pub execution_status: Option<String>,
    pub execution_time_ms: Option<i64>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub artifact_id: Option<ArtifactId>,
    pub artifact_type: Option<String>,
    pub artifact_title: Option<String>,
    pub artifact_kind: Option<String>,
    pub input_summary: Option<String>,
    pub is_builtin: bool,
    pub is_structured: bool,
    pub payload_bytes: Option<i32>,
    pub is_error: Option<bool>,
    pub error_message: Option<String>,
    pub request_id: Option<String>,
}

pub async fn list_conversation_tool_calls(
    pool: &PgPool,
    context_id: &ContextId,
    client_session_id: Option<&str>,
) -> Result<Vec<ConversationToolCallRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT t.tool_name, t.server_name, t.state AS "state!", t.source, t.execution_status,
                  t.execution_time_ms::bigint AS execution_time_ms, t.occurred_at,
                  t.artifact_id AS "artifact_id?: ArtifactId", t.artifact_type, t.artifact_title,
                  t.artifact_kind, t.input_summary, COALESCE(t.is_builtin, false) AS "is_builtin!",
                  COALESCE(t.is_structured, false) AS "is_structured!", t.payload_bytes,
                  t.is_error, t.error_message, t.request_id
           FROM tool_activity t
           WHERE t.context_id = $1 OR t.execution_context_id = $1
              OR ($2::text IS NOT NULL AND (t.execution_trace_id = $2 OR t.session_id = $2))
           ORDER BY t.occurred_at NULLS LAST
           LIMIT 500"#,
        context_id.as_str(),
        client_session_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ConversationToolCallRow {
            tool_name: r.tool_name,
            server_name: r.server_name,
            state: r.state,
            source: r.source,
            execution_status: r.execution_status,
            execution_time_ms: r.execution_time_ms,
            occurred_at: r.occurred_at,
            artifact_id: r.artifact_id,
            artifact_type: r.artifact_type,
            artifact_title: r.artifact_title,
            artifact_kind: r.artifact_kind,
            input_summary: r.input_summary,
            is_builtin: r.is_builtin,
            is_structured: r.is_structured,
            payload_bytes: r.payload_bytes,
            is_error: r.is_error,
            error_message: r.error_message,
            request_id: r.request_id,
        })
        .collect())
}
