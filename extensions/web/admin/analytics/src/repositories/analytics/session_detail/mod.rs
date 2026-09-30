//! Session-detail repository — drives `/admin/sessions/{id}`.
//!
//! A session groups every AI request, context, and trace produced by a single
//! interactive run. This module assembles the header row from
//! `plugin_session_summaries` (when present) plus an `ai_requests` rollup, and
//! returns the contexts/traces/requests that belong to the session.
//!
//! Three id shapes reach `/admin/sessions/{id}` and all resolve here: the
//! gateway's own `sess_…` id (`ai_requests.session_id`), the Claude Code
//! session uuid (`ai_requests.client_session_id`, which is also
//! `plugin_session_summaries.session_id`), and a hook-only session that never
//! produced gateway traffic. Every read matches `session_id OR
//! client_session_id`, so one page answers whichever id the link carried.
//!
//! `completed` is the one liveness fact the rows hold (a `SessionEnd` was
//! seen); live versus stale is the reader's call from `last_activity_at`. The
//! tool-call counts come from `tool_call_ledger`, so this page agrees with
//! Requests, Artifacts and Analysis.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{PluginId, SessionId, UserId};

mod lists;

pub use lists::{
    SessionContextRow, SessionRequestRow, SessionTraceRow, list_session_contexts,
    list_session_requests, list_session_traces,
};

#[derive(Debug, Clone)]
pub struct SessionHeader {
    pub session_id: SessionId,
    pub user_id: Option<UserId>,
    pub display_name: Option<String>,
    pub groups: Vec<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub completed: bool,
    pub model: Option<String>,
    pub plugin_id: Option<PluginId>,
    pub ai_title: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SessionKpis {
    pub request_count: i64,
    pub context_count: i64,
    pub trace_count: i64,
    pub error_count: i64,
    pub total_input_tokens: i64,
    pub total_cache_read_tokens: i64,
    pub total_cache_creation_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost_microdollars: i64,
    pub tool_call_count: i64,
    pub executed_count: i64,
    pub artifact_count: i64,
}

pub async fn find_session_header(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Option<SessionHeader>, sqlx::Error> {
    sqlx::query_as!(
        SessionHeader,
        r#"
        SELECT
            $1::text                             AS "session_id!: SessionId",
            COALESCE(s.user_id, r.user_id)       AS "user_id?: UserId",
            u.display_name                       AS "display_name?",
            ARRAY(SELECT ug.group_id FROM user_groups ug
                   WHERE ug.user_id = u.id)          AS "groups!: Vec<String>",
            COALESCE(s.started_at, r.first_seen) AS "started_at?",
            -- The last thing the session did from any vantage point: the
            -- last gateway request, the last hook event, or its end. A hook
            -- session that never wrote an `ended_at` still has a last
            -- activity: the moment it started.
            GREATEST(r.last_seen, h.last_event, COALESCE(s.ended_at, s.started_at))
                                                 AS "last_activity_at?",
            -- Only the end is a fact the row can state; a session with no
            -- end is live or stale by how recently it did anything, which
            -- the reader decides against the heartbeat window. A header built
            -- from requests alone has no session row at all, and the LEFT JOIN
            -- leaves this NULL rather than false, so it is coalesced: nothing
            -- to end is not completed.
            COALESCE(s.ended_at IS NOT NULL OR s.status = 'completed', FALSE) AS "completed!",
            COALESCE(s.model, r.model)           AS "model?",
            s.plugin_id                          AS "plugin_id?: PluginId",
            s.ai_title                           AS "ai_title?"
        FROM (
            SELECT
                COUNT(*)::bigint AS request_count,
                MAX(user_id) AS user_id,
                MIN(created_at) AS first_seen,
                MAX(created_at) AS last_seen,
                (ARRAY_AGG(model ORDER BY created_at DESC)
                    FILTER (WHERE model IS NOT NULL))[1] AS model
            FROM ai_requests
            WHERE session_id = $1 OR client_session_id = $1
        ) r
        LEFT JOIN plugin_session_summaries s ON s.session_id = $1
        LEFT JOIN LATERAL (
            SELECT MAX(e.created_at) AS last_event
            FROM plugin_usage_events e
            WHERE e.session_id = $1
        ) h ON TRUE
        LEFT JOIN users u ON u.id = COALESCE(s.user_id, r.user_id)
        WHERE r.request_count > 0 OR s.session_id IS NOT NULL
        LIMIT 1
        "#,
        session_id.as_str()
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_session_kpis(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<SessionKpis, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*)::bigint                                   AS "request_count!",
            COUNT(DISTINCT context_id)::bigint                 AS "context_count!",
            COUNT(DISTINCT trace_id)::bigint                   AS "trace_count!",
            COUNT(*) FILTER (WHERE status = 'failed')::bigint  AS "error_count!",
            COALESCE(SUM(input_tokens), 0)::bigint             AS "total_input_tokens!",
            COALESCE(SUM(cache_read_tokens), 0)::bigint        AS "total_cache_read_tokens!",
            COALESCE(SUM(cache_creation_tokens), 0)::bigint    AS "total_cache_creation_tokens!",
            COALESCE(SUM(output_tokens), 0)::bigint            AS "total_output_tokens!",
            COALESCE(SUM(cost_microdollars), 0)::bigint        AS "total_cost_microdollars!"
        FROM ai_requests
        WHERE session_id = $1 OR client_session_id = $1
        "#,
        session_id.as_str()
    )
    .fetch_one(pool)
    .await?;
    let tools = sqlx::query!(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE intent_id IS NOT NULL)::bigint          AS "tool_call_count!",
            COUNT(*) FILTER (WHERE mcp_execution_id IS NOT NULL)::bigint   AS "executed_count!",
            COUNT(*) FILTER (WHERE artifact_id IS NOT NULL)::bigint        AS "artifact_count!"
        FROM tool_call_ledger
        WHERE session_id = $1
        "#,
        session_id.as_str()
    )
    .fetch_one(pool)
    .await?;
    Ok(SessionKpis {
        tool_call_count: tools.tool_call_count,
        executed_count: tools.executed_count,
        artifact_count: tools.artifact_count,
        request_count: row.request_count,
        context_count: row.context_count,
        trace_count: row.trace_count,
        error_count: row.error_count,
        total_input_tokens: row.total_input_tokens,
        total_cache_read_tokens: row.total_cache_read_tokens,
        total_cache_creation_tokens: row.total_cache_creation_tokens,
        total_output_tokens: row.total_output_tokens,
        total_cost_microdollars: row.total_cost_microdollars,
    })
}
