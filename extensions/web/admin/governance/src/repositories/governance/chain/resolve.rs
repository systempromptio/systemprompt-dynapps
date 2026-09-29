//! Identifier resolution to a `session_id`.
//!
//! A pasted id is tried as a decision id, then as a request key, then as a
//! session id in each table that records one. A request with no session (an
//! internal `ai_service` call, or a client that sent no `x-session-id`) keys
//! its chain on its trace id, so the chain is that request alone. The hook
//! spine is empty for gateway-only runs, so a session id is looked up in the
//! decision and request tables too, not only in `plugin_usage_events`.

use sqlx::PgPool;
use systemprompt::identifiers::SessionId;

pub(super) async fn resolve_session_id(
    pool: &PgPool,
    id: &str,
) -> Result<Option<SessionId>, sqlx::Error> {
    if let Some(sid) = find_by_decision_id(pool, id).await? {
        return Ok(Some(sid));
    }
    if let Some(sid) = find_by_request_key_or_trace(pool, id).await? {
        return Ok(Some(sid));
    }
    if let Some(sid) = find_session_in_hook_spine(pool, id).await? {
        return Ok(Some(sid));
    }
    if let Some(sid) = find_session_in_decisions(pool, id).await? {
        return Ok(Some(sid));
    }
    find_session_in_requests(pool, id).await
}

async fn find_by_decision_id(pool: &PgPool, id: &str) -> Result<Option<SessionId>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"SELECT session_id as "session_id!: SessionId" FROM governance_decisions WHERE id = $1 LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    .map(|row| row.session_id))
}

async fn find_by_request_key_or_trace(
    pool: &PgPool,
    id: &str,
) -> Result<Option<SessionId>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"SELECT COALESCE(session_id, trace_id) as "session_id: SessionId" FROM ai_requests
          WHERE id = $1 OR request_id = $1 OR trace_id = $1
          ORDER BY session_id IS NULL
          LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    .and_then(|row| row.session_id))
}

async fn find_session_in_hook_spine(
    pool: &PgPool,
    id: &str,
) -> Result<Option<SessionId>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"SELECT session_id as "session_id!: SessionId" FROM plugin_usage_events WHERE session_id = $1 LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    .map(|row| row.session_id))
}

async fn find_session_in_decisions(
    pool: &PgPool,
    id: &str,
) -> Result<Option<SessionId>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"SELECT session_id as "session_id!: SessionId" FROM governance_decisions
           WHERE session_id = $1 AND session_id <> '' LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    .map(|row| row.session_id))
}

async fn find_session_in_requests(
    pool: &PgPool,
    id: &str,
) -> Result<Option<SessionId>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"SELECT session_id as "session_id: SessionId" FROM ai_requests
           WHERE session_id = $1 LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    .and_then(|row| row.session_id))
}
