//! The session page's three linked tables — contexts, traces and requests —
//! each capped at the newest 200 rows and matched on `session_id OR
//! client_session_id`; the legacy context is never listed as a context.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{AiRequestId, ContextId, SessionId, TraceId};

#[derive(Debug, Clone)]
pub struct SessionContextRow {
    pub context_id: ContextId,
    pub name: Option<String>,
    pub title: String,
    pub turn_count: i64,
    pub request_count: i64,
    pub last_request_at: Option<DateTime<Utc>>,
    pub model: Option<String>,
    pub total_input_tokens: i64,
    pub total_cache_read_tokens: i64,
    pub total_cache_creation_tokens: i64,
    pub total_output_tokens: i64,
    pub cost_microdollars: i64,
    pub error_count: i64,
}

#[derive(Debug, Clone)]
pub struct SessionTraceRow {
    pub trace_id: TraceId,
    pub request_count: i64,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub error_count: i64,
}

#[derive(Debug, Clone)]
pub struct SessionRequestRow {
    pub id: AiRequestId,
    pub context_id: Option<ContextId>,
    pub trace_id: Option<TraceId>,
    pub model: Option<String>,
    pub status: String,
    pub latency_ms: Option<i32>,
    pub input_tokens: Option<i32>,
    pub cache_read_tokens: Option<i32>,
    pub cache_creation_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub cost_microdollars: i64,
    pub created_at: DateTime<Utc>,
}

pub async fn list_session_contexts(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Vec<SessionContextRow>, sqlx::Error> {
    let legacy = ContextId::legacy();
    sqlx::query_as!(
        SessionContextRow,
        r#"
        SELECT
            r.context_id                         AS "context_id!: ContextId",
            c.name                               AS "name?",
            conversation_title(r.context_id, cr.client_session_id)
                                                 AS "title!",
            COALESCE(cr.turn_count, 0)::bigint   AS "turn_count!",
            COUNT(*)::bigint                     AS "request_count!",
            MAX(r.created_at)                    AS "last_request_at?",
            MAX(r.model)                         AS "model?",
            COALESCE(SUM(r.input_tokens), 0)::bigint     AS "total_input_tokens!",
            COALESCE(SUM(r.cache_read_tokens), 0)::bigint     AS "total_cache_read_tokens!",
            COALESCE(SUM(r.cache_creation_tokens), 0)::bigint AS "total_cache_creation_tokens!",
            COALESCE(SUM(r.output_tokens), 0)::bigint    AS "total_output_tokens!",
            COALESCE(SUM(r.cost_microdollars), 0)::bigint AS "cost_microdollars!",
            COUNT(*) FILTER (WHERE r.status = 'failed')::bigint AS "error_count!"
        FROM ai_requests r
        LEFT JOIN user_contexts c ON c.context_id = r.context_id
        LEFT JOIN conversation_metrics_for(ARRAY(
            SELECT DISTINCT context_id::text FROM ai_requests
            WHERE session_id = $1 OR client_session_id = $1
        )) cr ON cr.context_id = r.context_id
        WHERE (r.session_id = $1 OR r.client_session_id = $1) AND r.context_id <> $2
        GROUP BY r.context_id, c.name, cr.client_session_id, cr.turn_count
        ORDER BY MAX(r.created_at) DESC
        LIMIT 200
        "#,
        session_id.as_str(),
        legacy.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn list_session_traces(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Vec<SessionTraceRow>, sqlx::Error> {
    sqlx::query_as!(
        SessionTraceRow,
        r#"
        SELECT
            trace_id                                            AS "trace_id!: TraceId",
            COUNT(*)::bigint                                    AS "request_count!",
            MIN(created_at)                                     AS "started_at?",
            MAX(COALESCE(completed_at, created_at))             AS "ended_at?",
            COUNT(*) FILTER (WHERE status = 'failed')::bigint   AS "error_count!"
        FROM ai_requests
        WHERE (session_id = $1 OR client_session_id = $1) AND trace_id IS NOT NULL
        GROUP BY trace_id
        ORDER BY MIN(created_at) DESC
        LIMIT 200
        "#,
        session_id.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn list_session_requests(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Vec<SessionRequestRow>, sqlx::Error> {
    let legacy = ContextId::legacy();
    sqlx::query_as!(
        SessionRequestRow,
        r#"
        SELECT
            id                                  AS "id!: AiRequestId",
            NULLIF(context_id, $2)              AS "context_id?: ContextId",
            trace_id                            AS "trace_id?: TraceId",
            model                               AS "model?",
            status                              AS "status!",
            latency_ms                          AS "latency_ms?",
            input_tokens                        AS "input_tokens?",
            cache_read_tokens                   AS "cache_read_tokens?",
            cache_creation_tokens               AS "cache_creation_tokens?",
            output_tokens                       AS "output_tokens?",
            cost_microdollars                   AS "cost_microdollars!",
            created_at                          AS "created_at!"
        FROM ai_requests
        WHERE session_id = $1 OR client_session_id = $1
        ORDER BY created_at DESC
        LIMIT 200
        "#,
        session_id.as_str(),
        legacy.as_str()
    )
    .fetch_all(pool)
    .await
}
