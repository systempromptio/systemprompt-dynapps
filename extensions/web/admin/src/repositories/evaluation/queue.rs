//! The evaluation outbox: which closed sessions still owe a score.
//!
//! A session is closed when its host said so (`SessionEnd`), or — only for
//! hosts whose adapter has no end-of-session event — when nothing has arrived
//! for the configured quiet period. Rows are leased with
//! `FOR UPDATE SKIP LOCKED` so several nodes can drain the queue together,
//! and a lease that expires is simply re-claimed.

use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

use super::CriterionName;

// Why: hosts whose bridge adapter has no end-of-session signal, so the queue
// may close their sessions on silence. Claude Code and OpenCode both send
// SessionEnd and are never closed this way.
pub const HOSTS_WITHOUT_SESSION_END: &[&str] = &["codex", "codex-cli"];

#[derive(Debug, Clone, Copy)]
pub struct CloseParams<'a> {
    pub quiet_minutes: u32,
    pub min_user_prompts: u32,
    pub lookback_hours: u32,
    pub criteria: &'a [CriterionName],
}

#[derive(Debug, Clone)]
pub struct EvaluationLease {
    pub session_id: SessionId,
    pub criterion: CriterionName,
    pub user_id: UserId,
    pub host: String,
    pub attempts: i32,
    pub token: String,
}

pub async fn insert_closed_sessions(
    pool: &PgPool,
    params: &CloseParams<'_>,
) -> Result<u64, sqlx::Error> {
    let criteria: Vec<String> = params
        .criteria
        .iter()
        .map(|c| c.as_str().to_owned())
        .collect();
    let quiet = i32::try_from(params.quiet_minutes).unwrap_or(i32::MAX);
    let lookback = i32::try_from(params.lookback_hours).unwrap_or(i32::MAX);
    let min_prompts = i64::from(params.min_user_prompts);
    let result = sqlx::query!(
        r#"INSERT INTO evaluation_queue (session_id, criterion, user_id, host, closed_at, closed_reason)
           SELECT s.session_id, c.criterion, s.user_id,
                  COALESCE(NULLIF(s.client_source, ''), 'unknown'),
                  COALESCE(s.ended_at, s.updated_at),
                  CASE WHEN s.ended_at IS NOT NULL THEN 'session_end' ELSE 'quiet' END
             FROM plugin_session_summaries s
             JOIN users u ON u.id = s.user_id
             CROSS JOIN unnest($1::text[]) AS c(criterion)
            WHERE s.started_at > clock_timestamp() - make_interval(hours => $2)
              AND COALESCE(s.user_prompts, s.prompts) >= $3
              AND (s.ended_at IS NOT NULL
                   OR (s.client_source = ANY($4::text[])
                       AND s.updated_at < clock_timestamp() - make_interval(mins => $5)))
              AND NOT EXISTS (SELECT 1 FROM evaluation_runs r
                               WHERE r.session_id = s.session_id AND r.criterion = c.criterion
                                 AND r.status = 'succeeded')
           ON CONFLICT DO NOTHING"#,
        &criteria,
        lookback,
        min_prompts,
        &HOSTS_WITHOUT_SESSION_END.iter().map(|h| (*h).to_owned()).collect::<Vec<_>>(),
        quiet,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub async fn claim_leases(
    pool: &PgPool,
    limit: u32,
    lease_seconds: u32,
) -> Result<Vec<EvaluationLease>, sqlx::Error> {
    let token = uuid::Uuid::new_v4().to_string();
    let rows = sqlx::query!(
        r#"WITH pending AS (
             SELECT session_id, criterion FROM evaluation_queue
              WHERE pending AND next_attempt <= clock_timestamp()
                AND (lease_until IS NULL OR lease_until <= clock_timestamp())
              ORDER BY next_attempt, session_id, criterion
              LIMIT $1 FOR UPDATE SKIP LOCKED)
           UPDATE evaluation_queue q
              SET lease_token = $2,
                  lease_until = clock_timestamp() + make_interval(secs => $3),
                  attempts = q.attempts + 1,
                  updated_at = clock_timestamp()
             FROM pending
            WHERE q.session_id = pending.session_id AND q.criterion = pending.criterion
           RETURNING q.session_id, q.criterion, q.user_id, q.host, q.attempts"#,
        i64::from(limit),
        token,
        f64::from(lease_seconds),
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            Some(EvaluationLease {
                session_id: SessionId::new(row.session_id),
                criterion: CriterionName::parse(&row.criterion)?,
                user_id: UserId::new(row.user_id),
                host: row.host,
                attempts: row.attempts,
                token: token.clone(),
            })
        })
        .collect())
}

pub async fn complete_lease(pool: &PgPool, lease: &EvaluationLease) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE evaluation_queue SET pending = false, lease_token = NULL, lease_until = NULL,
                last_error = NULL, updated_at = clock_timestamp()
          WHERE session_id = $1 AND criterion = $2 AND lease_token = $3",
        lease.session_id.as_str(),
        lease.criterion.as_str(),
        lease.token,
    )
    .execute(pool)
    .await?;
    Ok(())
}

// Why: releases the lease with backoff; after `max_attempts` the row stays
// with `pending = false` and its `last_error`, so a dead letter is visible
// rather than retried forever.
pub async fn retry_lease(
    pool: &PgPool,
    lease: &EvaluationLease,
    error: &str,
    max_attempts: u32,
) -> Result<(), sqlx::Error> {
    let exhausted = u32::try_from(lease.attempts).unwrap_or(u32::MAX) >= max_attempts;
    sqlx::query!(
        "UPDATE evaluation_queue
            SET pending = $4, lease_token = NULL, lease_until = NULL,
                next_attempt = clock_timestamp() + least(attempts, 60) * interval '30 seconds',
                last_error = left($5, 2000), updated_at = clock_timestamp()
          WHERE session_id = $1 AND criterion = $2 AND lease_token = $3",
        lease.session_id.as_str(),
        lease.criterion.as_str(),
        lease.token,
        !exhausted,
        error,
    )
    .execute(pool)
    .await?;
    Ok(())
}
