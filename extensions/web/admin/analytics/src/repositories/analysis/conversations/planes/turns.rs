//! The turn ledger of one conversation: one row per gateway request, in order,
//! joined from `conversation_requests` and `ai_requests` (capped at 500).

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::ContextId;

/// One gateway request of the conversation, in order.
#[derive(Debug, Clone)]
pub struct ConversationTurnRow {
    pub request_id: String,
    pub created_at: DateTime<Utc>,
    pub effective_kind: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub requested_model: Option<String>,
    pub route_match: Option<String>,
    pub status: String,
    pub finish_reason: Option<String>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub reasoning_tokens: i64,
    pub cost_microdollars: i64,
    pub latency_ms: Option<i32>,
    pub upstream_latency_ms: Option<i32>,
    pub is_streaming: bool,
    pub max_tokens: Option<i32>,
    pub tool_calls: i64,
    pub tool_names: Vec<String>,
    pub safety_findings: i64,
    pub safety_blocked: i64,
    pub error_message: Option<String>,
}

pub async fn list_conversation_turns(
    pool: &PgPool,
    context_id: &ContextId,
) -> Result<Vec<ConversationTurnRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT cr.id AS "request_id!", cr.created_at AS "created_at!",
                  cr.effective_kind AS "effective_kind!", cr.provider, cr.model, ar.requested_model,
                  ar.route_match, cr.status AS "status!", ar.finish_reason,
                  COALESCE(cr.input_tokens, 0)::bigint AS "input_tokens!",
                  COALESCE(cr.output_tokens, 0)::bigint AS "output_tokens!",
                  COALESCE(ar.cache_read_tokens, 0)::bigint AS "cache_read_tokens!",
                  COALESCE(ar.cache_creation_tokens, 0)::bigint AS "cache_creation_tokens!",
                  COALESCE(ar.reasoning_tokens, 0)::bigint AS "reasoning_tokens!",
                  cr.cost_microdollars AS "cost_microdollars!", cr.latency_ms, ar.upstream_latency_ms,
                  COALESCE(ar.is_streaming, false) AS "is_streaming!", cr.max_tokens,
                  (SELECT COUNT(*)::bigint FROM ai_request_tool_calls t WHERE t.request_id = cr.id) AS "tool_calls!",
                  ARRAY(SELECT t.tool_name FROM ai_request_tool_calls t WHERE t.request_id = cr.id
                        ORDER BY t.sequence_number) AS "tool_names!",
                  (SELECT COUNT(*)::bigint FROM ai_safety_findings s WHERE s.ai_request_id = cr.id) AS "safety_findings!",
                  (SELECT COUNT(*)::bigint FROM ai_safety_findings s WHERE s.ai_request_id = cr.id AND s.blocked) AS "safety_blocked!",
                  ar.error_message
           FROM conversation_requests cr
           JOIN ai_requests ar ON ar.id = cr.id
           WHERE cr.context_id = $1
           ORDER BY cr.created_at, cr.id
           LIMIT 500"#,
        context_id.as_str()
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ConversationTurnRow {
            request_id: r.request_id,
            created_at: r.created_at,
            effective_kind: r.effective_kind,
            provider: r.provider,
            model: r.model,
            requested_model: r.requested_model,
            route_match: r.route_match,
            status: r.status,
            finish_reason: r.finish_reason,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            cache_read_tokens: r.cache_read_tokens,
            cache_creation_tokens: r.cache_creation_tokens,
            reasoning_tokens: r.reasoning_tokens,
            cost_microdollars: r.cost_microdollars,
            latency_ms: r.latency_ms,
            upstream_latency_ms: r.upstream_latency_ms,
            is_streaming: r.is_streaming,
            max_tokens: r.max_tokens,
            tool_calls: r.tool_calls,
            tool_names: r.tool_names,
            safety_findings: r.safety_findings,
            safety_blocked: r.safety_blocked,
            error_message: r.error_message,
        })
        .collect())
}
