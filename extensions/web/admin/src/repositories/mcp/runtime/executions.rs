//! Tool-call traffic per MCP server: the rollup and the per-tool split.
//!
//! `mcp_tool_executions` is the only record of what a server actually did, so
//! every figure the MCP pages show about behaviour comes from here. Two rules
//! shape every query:
//!
//! - A call is counted once, from the server's side. The proxy and the
//!   in-process executor each write one row per call (`source` in `in_process`,
//!   `proxy`); the client's completion hook writes a second row for the same
//!   call (`hook_*`) that carries no duration and no output. Calls, errors and
//!   latency read the server-observed rows; the hook rows are reported beside
//!   them as *attested* — how many calls the client also confirmed — never
//!   added to them.
//! - Failures are counted separately from timeouts because they are different
//!   operator problems: a failing tool is a bug, a timing-out one is capacity.

use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct McpServerActivity {
    pub server_name: String,
    pub calls: i64,
    pub succeeded: i64,
    pub failures: i64,
    pub timeouts: i64,
    pub attested: i64,
    pub distinct_users: i64,
    pub distinct_sessions: i64,
    pub distinct_tools: i64,
    pub avg_ms: Option<f64>,
    pub p95_ms: Option<f64>,
    pub payload_bytes: i64,
    pub secret_redactions: i64,
    pub last_call_at: Option<DateTime<Utc>>,
    pub prior_calls: i64,
}

// Why: Per-server call volume over a window, with the preceding window of equal
// length beside it so every headline number can carry a delta.
pub async fn list_mcp_server_activity(
    pool: &PgPool,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<McpServerActivity>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"WITH windowed AS (
             SELECT x.*, a.payload_bytes, a.secret_redactions,
                    (x.source IN ('in_process', 'proxy')) AS seen
               FROM mcp_tool_executions x
               LEFT JOIN (
                 SELECT mcp_execution_id,
                        SUM(payload_bytes)::BIGINT AS payload_bytes,
                        SUM(secret_redactions)::BIGINT AS secret_redactions
                   FROM mcp_artifacts GROUP BY mcp_execution_id
               ) a ON a.mcp_execution_id = x.mcp_execution_id
              WHERE x.started_at >= $1 AND x.started_at < $2
           ),
           prior AS (
             SELECT server_name, COUNT(*)::BIGINT AS calls
               FROM mcp_tool_executions
              WHERE started_at >= $1 - ($2 - $1) AND started_at < $1
                AND source IN ('in_process', 'proxy')
              GROUP BY server_name
           )
           SELECT
             w.server_name AS "server_name!",
             COUNT(*) FILTER (WHERE w.seen)::BIGINT AS "calls!",
             COUNT(*) FILTER (WHERE w.seen AND w.status = 'success')::BIGINT AS "succeeded!",
             COUNT(*) FILTER (WHERE w.seen AND w.status = 'failed')::BIGINT AS "failures!",
             COUNT(*) FILTER (WHERE w.seen AND w.status = 'timeout')::BIGINT AS "timeouts!",
             COUNT(*) FILTER (WHERE w.source LIKE 'hook_%')::BIGINT AS "attested!",
             COUNT(DISTINCT w.user_id) FILTER (WHERE w.seen)::BIGINT AS "distinct_users!",
             COUNT(DISTINCT w.session_id) FILTER (WHERE w.seen)::BIGINT AS "distinct_sessions!",
             COUNT(DISTINCT w.tool_name) FILTER (WHERE w.seen)::BIGINT AS "distinct_tools!",
             AVG(w.execution_time_ms) FILTER (WHERE w.seen)::FLOAT8 AS "avg_ms?",
             PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY w.execution_time_ms)
               FILTER (WHERE w.seen)::FLOAT8 AS "p95_ms?",
             COALESCE(SUM(w.payload_bytes) FILTER (WHERE w.seen), 0)::BIGINT AS "payload_bytes!",
             COALESCE(SUM(w.secret_redactions) FILTER (WHERE w.seen), 0)::BIGINT
               AS "secret_redactions!",
             MAX(w.started_at) FILTER (WHERE w.seen) AS "last_call_at?",
             COALESCE(MAX(p.calls), 0)::BIGINT AS "prior_calls!"
           FROM windowed w
           LEFT JOIN prior p ON p.server_name = w.server_name
          GROUP BY w.server_name
          ORDER BY COUNT(*) FILTER (WHERE w.seen) DESC"#,
        from,
        to
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| McpServerActivity {
            server_name: r.server_name,
            calls: r.calls,
            succeeded: r.succeeded,
            failures: r.failures,
            timeouts: r.timeouts,
            attested: r.attested,
            distinct_users: r.distinct_users,
            distinct_sessions: r.distinct_sessions,
            distinct_tools: r.distinct_tools,
            avg_ms: r.avg_ms,
            p95_ms: r.p95_ms,
            payload_bytes: r.payload_bytes,
            secret_redactions: r.secret_redactions,
            last_call_at: r.last_call_at,
            prior_calls: r.prior_calls,
        })
        .collect())
}

#[derive(Debug, Clone)]
pub struct McpToolStat {
    pub server_name: String,
    pub tool_name: String,
    pub calls: i64,
    pub failures: i64,
    pub attested: i64,
    pub distinct_users: i64,
    pub avg_ms: Option<f64>,
    pub p95_ms: Option<f64>,
    pub max_ms: Option<i32>,
    pub payload_bytes: i64,
    pub last_call_at: Option<DateTime<Utc>>,
}

// Why: The tools served in the window, busiest first — for one server, or for
// the whole fleet when `server_name` is `None` (the export).
pub async fn list_mcp_tool_stats(
    pool: &PgPool,
    server_name: Option<&str>,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    limit: i64,
) -> Result<Vec<McpToolStat>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"WITH windowed AS (
             SELECT x.*, a.payload_bytes
               FROM mcp_tool_executions x
               LEFT JOIN (
                 SELECT mcp_execution_id, SUM(payload_bytes)::BIGINT AS payload_bytes
                   FROM mcp_artifacts GROUP BY mcp_execution_id
               ) a ON a.mcp_execution_id = x.mcp_execution_id
              WHERE ($1::TEXT IS NULL OR x.server_name = $1)
                AND x.started_at >= $2 AND x.started_at < $3
           )
           SELECT
             server_name AS "server_name!",
             tool_name AS "tool_name!",
             COUNT(*) FILTER (WHERE source IN ('in_process', 'proxy'))::BIGINT AS "calls!",
             COUNT(*) FILTER (
               WHERE source IN ('in_process', 'proxy') AND status IN ('failed', 'timeout')
             )::BIGINT AS "failures!",
             COUNT(*) FILTER (WHERE source LIKE 'hook_%')::BIGINT AS "attested!",
             COUNT(DISTINCT user_id) FILTER (
               WHERE source IN ('in_process', 'proxy')
             )::BIGINT AS "distinct_users!",
             AVG(execution_time_ms) FILTER (
               WHERE source IN ('in_process', 'proxy')
             )::FLOAT8 AS "avg_ms?",
             PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY execution_time_ms)
               FILTER (WHERE source IN ('in_process', 'proxy'))::FLOAT8 AS "p95_ms?",
             MAX(execution_time_ms) FILTER (
               WHERE source IN ('in_process', 'proxy')
             ) AS "max_ms?",
             COALESCE(SUM(payload_bytes) FILTER (
               WHERE source IN ('in_process', 'proxy')
             ), 0)::BIGINT AS "payload_bytes!",
             MAX(started_at) FILTER (
               WHERE source IN ('in_process', 'proxy')
             ) AS "last_call_at?"
           FROM windowed
          GROUP BY server_name, tool_name
          ORDER BY COUNT(*) FILTER (WHERE source IN ('in_process', 'proxy')) DESC, tool_name
          LIMIT $4"#,
        server_name,
        from,
        to,
        limit
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| McpToolStat {
            server_name: r.server_name,
            tool_name: r.tool_name,
            calls: r.calls,
            failures: r.failures,
            attested: r.attested,
            distinct_users: r.distinct_users,
            avg_ms: r.avg_ms,
            p95_ms: r.p95_ms,
            max_ms: r.max_ms,
            payload_bytes: r.payload_bytes,
            last_call_at: r.last_call_at,
        })
        .collect())
}
