//! Resolves the id an operator pasted or clicked — a `trace_id`, a
//! `session_id`, a hook-only session, or an MCP execution — to the key that
//! every span source is queried by.
//!
//! That key is normally the `session_id` shared by the span tables. An MCP
//! execution is the one span that may carry a `trace_id` no other table
//! knows (core's MCP client sends no `x-trace-id`, so a proxied call's trace
//! id exists in `mcp_tool_executions` alone) and may have no session at all;
//! [`execution_trace_key`] states which of its ids stands in as the key.

use sqlx::PgPool;
use systemprompt::identifiers::{McpExecutionId, SessionId, TraceId};

pub async fn resolve_trace_session(
    pool: &PgPool,
    id: &str,
) -> Result<Option<SessionId>, sqlx::Error> {
    // Why: a gateway request that arrived with no session (an internal
    // `ai_service` call, or a client that sent no `x-session-id`) still has
    // a trace; its trace id stands in as the key, the same rule as
    // [`execution_trace_key`], and the request span source matches on it.
    if let Some(row) = sqlx::query!(
        r#"SELECT COALESCE(session_id, trace_id) AS "session_id!: SessionId"
           FROM ai_requests
           WHERE trace_id = $1
           ORDER BY session_id IS NULL
           LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(row.session_id));
    }

    // Why: a pasted Claude Code session uuid is the `client_session_id` of
    // the gateway rows; it lands on the bridge session those rows and the
    // hook rows are folded into on the list.
    if let Some(row) = sqlx::query!(
        r#"SELECT session_id AS "session_id!: SessionId"
           FROM ai_requests
           WHERE client_session_id = $1 AND session_id IS NOT NULL
           ORDER BY created_at DESC
           LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(row.session_id));
    }

    if let Some(row) = sqlx::query!(
        r#"SELECT session_id AS "session_id!: SessionId"
           FROM governance_decisions
           WHERE session_id = $1
           LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(row.session_id));
    }

    if let Some(row) = sqlx::query!(
        r#"SELECT session_id AS "session_id!: SessionId"
           FROM ai_requests
           WHERE session_id = $1
           LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(row.session_id));
    }

    // Why: hook-only traces exist solely in plugin_usage_events; without this
    // arm the trace list links them but the detail page answers 404.
    if let Some(row) = sqlx::query!(
        r#"SELECT session_id AS "session_id!: SessionId"
           FROM plugin_usage_events
           WHERE session_id = $1
           LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(row.session_id));
    }

    find_execution_key(pool, id).await
}

// Why: an in-process or proxied tool call lands only in
// `mcp_tool_executions`; its `trace_id` and `mcp_execution_id` are both
// linked from the MCP server page, so both must resolve here.
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

// Why: the key an execution's spans are queried by — its session when it has
// one, else its trace id, else the execution id itself. The ledger span
// source matches all three, so whichever stands in still finds the execution.
pub fn execution_trace_key(
    session_id: Option<SessionId>,
    trace_id: Option<&TraceId>,
    execution_id: &McpExecutionId,
) -> SessionId {
    session_id
        .unwrap_or_else(|| SessionId::new(trace_id.map_or(execution_id.as_str(), TraceId::as_str)))
}
