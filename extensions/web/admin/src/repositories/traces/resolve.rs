//! Resolves the id an operator pasted or clicked — a `trace_id`, a
//! `session_id`, or a hook-only session — to the `session_id` that keys
//! every span table.

use sqlx::PgPool;
use systemprompt::identifiers::SessionId;

pub async fn resolve_trace_session(
    pool: &PgPool,
    id: &str,
) -> Result<Option<SessionId>, sqlx::Error> {
    if let Some(row) = sqlx::query!(
        r#"SELECT session_id AS "session_id!: SessionId"
           FROM ai_requests
           WHERE trace_id = $1 AND session_id IS NOT NULL
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

    Ok(None)
}
