//! Tool-call spans from the ledger: in-process and proxied MCP executions,
//! each joined to the artifact it produced.
//!
//! `mcp_tool_executions` is the one span table the gateway trace id may live
//! in alone — core's MCP client sends no `x-trace-id`, so a proxied call's
//! trace id reaches no other row. The ledger coalesces the trace id across
//! intent, execution and result, and the query below accepts a session, a
//! trace id or an execution id as its key so that every id the MCP server
//! page links resolves to these spans.

use sqlx::PgPool;
use systemprompt::identifiers::{ArtifactId, ContextId, SessionId, UserId};

use super::spans::format_identity;
use super::{Span, SpanKind, SpanStatus};

// Why: in-process and proxied tool calls are never reported by a hook, so
// they exist only in `mcp_tool_executions`; the ledger joins each to its
// artifact and coalesces the trace id across intent, execution and result.
// Hook-sourced executions are the same calls `list_event_spans` already
// renders and are left out rather than shown twice. The key may be a session,
// a trace id or an execution id (see `resolve::execution_trace_key`).
pub(super) async fn list_ledger_spans(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Vec<Span>, sqlx::Error> {
    let calls = sqlx::query!(
        r#"SELECT
            l.mcp_execution_id  AS "mcp_execution_id!",
            l.tool_name         AS "tool_name!",
            l.server_name       AS "server_name!",
            l.source            AS "source!",
            l.execution_status  AS "execution_status!",
            l.error_message,
            l.executed_at       AS "executed_at!",
            l.completed_at,
            l.execution_time_ms,
            l.trace_id,
            l.context_id        AS "context_id: ContextId",
            l.artifact_id       AS "artifact_id: ArtifactId",
            COALESCE(l.is_error, FALSE) AS "is_error!",
            l.user_id           AS "user_id!: UserId",
            COALESCE(u.display_name, u.full_name, u.name, u.email, l.user_id) AS "user_label!"
        FROM tool_call_ledger l
        LEFT JOIN users u ON u.id = l.user_id
        WHERE l.mcp_execution_id IS NOT NULL
          AND l.source NOT IN ('hook_claude_code', 'hook_opencode')
          AND (l.session_id = $1
               OR l.trace_id = $1
               OR l.mcp_execution_id = $1
               OR l.session_id IN (
                   SELECT DISTINCT client_session_id FROM ai_requests
                   WHERE session_id = $1 AND client_session_id IS NOT NULL))
        ORDER BY l.executed_at ASC"#,
        session_id.as_str(),
    )
    .fetch_all(pool)
    .await?;

    Ok(calls
        .into_iter()
        .map(|c| {
            let started = c.executed_at;
            let ended = c.completed_at.unwrap_or_else(|| {
                started
                    + chrono::Duration::milliseconds(
                        i64::from(c.execution_time_ms.unwrap_or(0)).max(0),
                    )
            });
            Span {
                id: c.mcp_execution_id,
                kind: SpanKind::Tool,
                name: c.tool_name,
                started_at: started,
                ended_at: ended,
                duration_ms: (ended - started).num_milliseconds().max(0),
                status: execution_status(&c.execution_status, c.is_error),
                identity_label: Some(format_identity(&c.user_label, None, None)),
                provider: None,
                model: None,
                cost_microdollars: None,
                latency_ms: c.execution_time_ms.map(i64::from),
                input_tokens: None,
                output_tokens: None,
                cache_read_tokens: None,
                error_message: c.error_message.filter(|e| !e.is_empty()),
                trace_id: c.trace_id,
                context_id: c.context_id,
                user_id: c.user_id,
                client_kind: None,
                policy: Some(execution_label(&c.server_name, &c.source)),
                decision: None,
                reason: None,
                stages: Vec::new(),
                artifact_id: c.artifact_id,
            }
        })
        .collect())
}

// Why: the execution's own verdict, except that a tool which ran to
// completion and returned an error result is an error.
pub fn execution_status(status: &str, artifact_is_error: bool) -> SpanStatus {
    match status {
        "pending" => SpanStatus::Pending,
        "success" if !artifact_is_error => SpanStatus::Ok,
        _ => SpanStatus::Error,
    }
}

// Why: the second line under an execution span — the server that ran it and
// where the platform saw it (`in_process`, `proxy`, `gateway`).
pub fn execution_label(server_name: &str, source: &str) -> String {
    format!("{server_name} · {source}")
}
