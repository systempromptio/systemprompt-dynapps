//! Resolves the id an operator pasted or clicked — a `trace_id`, a
//! `session_id`, a hook-only session, or an MCP execution — to the key that
//! every span source is queried by.
//!
//! That key is normally the `session_id` shared by the span tables. An MCP
//! execution is the one span that may carry a `trace_id` no other table
//! knows (core's MCP client sends no `x-trace-id`, so a proxied call's trace
//! id exists in `mcp_tool_executions` alone) and may have no session at all;
//! [`execution_trace_key`] states which of its ids stands in as the key.
//! A gateway request with no session follows the same rule: its trace id
//! stands in, and the request span source matches on it.

use sqlx::PgPool;
use systemprompt::identifiers::{McpExecutionId, SessionId, TraceId, UserId};

pub async fn resolve_trace_session(
    pool: &PgPool,
    id: &str,
) -> Result<Option<SessionId>, sqlx::Error> {
    if let Some(key) = find_request_key(pool, id).await? {
        return Ok(Some(key));
    }
    if let Some(key) = find_hook_key(pool, id).await? {
        return Ok(Some(key));
    }
    find_execution_key(pool, id).await
}

async fn find_request_key(pool: &PgPool, id: &str) -> Result<Option<SessionId>, sqlx::Error> {
    let by_trace = sqlx::query!(
        r#"SELECT COALESCE(session_id, trace_id) AS "session_id!: SessionId"
           FROM ai_requests
           WHERE trace_id = $1
           ORDER BY session_id IS NULL
           LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?;
    if let Some(row) = by_trace {
        return Ok(Some(row.session_id));
    }
    let by_client_session = sqlx::query!(
        r#"SELECT session_id AS "session_id!: SessionId"
           FROM ai_requests
           WHERE client_session_id = $1 AND session_id IS NOT NULL
           ORDER BY created_at DESC
           LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(by_client_session.map(|row| row.session_id))
}

async fn find_hook_key(pool: &PgPool, id: &str) -> Result<Option<SessionId>, sqlx::Error> {
    let by_governance = sqlx::query!(
        r#"SELECT session_id AS "session_id!: SessionId"
           FROM governance_decisions
           WHERE session_id = $1
           LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?;
    if let Some(row) = by_governance {
        return Ok(Some(row.session_id));
    }
    let by_request = sqlx::query!(
        r#"SELECT session_id AS "session_id!: SessionId"
           FROM ai_requests
           WHERE session_id = $1
           LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?;
    if let Some(row) = by_request {
        return Ok(Some(row.session_id));
    }
    let by_hook_event = sqlx::query!(
        r#"SELECT session_id AS "session_id!: SessionId"
           FROM plugin_usage_events
           WHERE session_id = $1
           LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(by_hook_event.map(|row| row.session_id))
}

pub async fn find_trace_owner(
    pool: &PgPool,
    key: &SessionId,
) -> Result<Option<UserId>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT owner AS "owner!: UserId" FROM (
               SELECT user_id AS owner, 0 AS rank FROM ai_requests
               WHERE (session_id = $1 OR trace_id = $1) AND user_id IS NOT NULL
               UNION ALL
               SELECT user_id, 1 FROM plugin_usage_events
               WHERE session_id = $1 AND user_id IS NOT NULL
               UNION ALL
               SELECT user_id, 2 FROM governance_decisions
               WHERE session_id = $1 AND user_id IS NOT NULL
           ) owners
           ORDER BY rank
           LIMIT 1"#,
        key.as_str(),
    )
    .fetch_optional(pool)
    .await
}

async fn find_execution_key(pool: &PgPool, id: &str) -> Result<Option<SessionId>, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT
            mcp_execution_id AS "mcp_execution_id!: McpExecutionId",
            session_id       AS "session_id: SessionId",
            trace_id         AS "trace_id: TraceId"
           FROM mcp_tool_executions
           WHERE trace_id = $1 OR mcp_execution_id = $1
           ORDER BY session_id IS NULL, started_at DESC
           LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| execution_trace_key(r.session_id, r.trace_id.as_ref(), &r.mcp_execution_id)))
}

pub fn execution_trace_key(
    session_id: Option<SessionId>,
    trace_id: Option<&TraceId>,
    execution_id: &McpExecutionId,
) -> SessionId {
    session_id
        .unwrap_or_else(|| SessionId::new(trace_id.map_or(execution_id.as_str(), TraceId::as_str)))
}
