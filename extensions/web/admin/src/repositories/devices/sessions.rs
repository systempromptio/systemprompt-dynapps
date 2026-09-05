//! Bridge sessions: one row per desktop client that has ever called home.
//!
//! A session is never deleted, only left to go quiet, so the listing is the
//! whole enrolment history and staleness is the only thing separating a fleet
//! from an archive. The stale predicate is therefore evaluated in SQL and
//! returned on the row, so the filter, the count and the badge are one
//! expression rather than three that have to be kept in step.
//!
//! Ordering is a bound parameter rather than an interpolated column name: the
//! sort key arrives from the query string, and a static statement that cannot
//! be reshaped by its own arguments is the only form in which that is safe.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

use super::STALE_AFTER_DAYS;

#[derive(Debug, Clone)]
pub struct BridgeSessionRow {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub user_name: String,
    pub user_email: String,
    pub hostname: String,
    pub os: String,
    pub bridge_version: String,
    pub started_at: DateTime<Utc>,
    pub last_heartbeat_at: DateTime<Utc>,
    pub forwarded_total: i64,
    pub tokens_in_total: i64,
    pub tokens_out_total: i64,
    pub is_stale: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct SessionQuery<'a> {
    pub stale_only: bool,
    pub sort: &'a str,
    pub dir: &'a str,
    pub limit: i64,
    pub offset: i64,
}

pub async fn list_bridge_sessions_paged(
    pool: &PgPool,
    query: SessionQuery<'_>,
) -> Result<(Vec<BridgeSessionRow>, i64), sqlx::Error> {
    let days = i32::try_from(STALE_AFTER_DAYS).unwrap_or(7);
    let rows = sqlx::query_as!(
        BridgeSessionRow,
        r#"SELECT b.session_id AS "session_id!: SessionId", b.user_id AS "user_id!: UserId",
                  u.name AS "user_name!", u.email AS "user_email!",
                  b.hostname AS "hostname!", b.os AS "os!",
                  b.bridge_version AS "bridge_version!",
                  b.started_at AS "started_at!",
                  b.last_heartbeat_at AS "last_heartbeat_at!",
                  b.forwarded_total AS "forwarded_total!",
                  b.tokens_in_total AS "tokens_in_total!",
                  b.tokens_out_total AS "tokens_out_total!",
                  (b.last_heartbeat_at < NOW() - make_interval(days => $1))
                      AS "is_stale!"
             FROM bridge_sessions b
             JOIN users u ON u.id = b.user_id
            WHERE NOT $2::BOOLEAN
               OR b.last_heartbeat_at < NOW() - make_interval(days => $1)
            ORDER BY
              CASE WHEN $4::TEXT = 'desc' THEN
                CASE $3::TEXT WHEN 'started' THEN EXTRACT(EPOCH FROM b.started_at)
                        WHEN 'forwarded' THEN b.forwarded_total::FLOAT8
                        WHEN 'tokens' THEN
                             (b.tokens_in_total + b.tokens_out_total)::FLOAT8
                        ELSE EXTRACT(EPOCH FROM b.last_heartbeat_at) END
              END DESC NULLS LAST,
              CASE WHEN $4::TEXT = 'asc' THEN
                CASE $3::TEXT WHEN 'started' THEN EXTRACT(EPOCH FROM b.started_at)
                        WHEN 'forwarded' THEN b.forwarded_total::FLOAT8
                        WHEN 'tokens' THEN
                             (b.tokens_in_total + b.tokens_out_total)::FLOAT8
                        ELSE EXTRACT(EPOCH FROM b.last_heartbeat_at) END
              END ASC NULLS LAST,
              CASE WHEN $3::TEXT = 'host' AND $4::TEXT = 'desc' THEN b.hostname END DESC,
              CASE WHEN $3::TEXT = 'host' AND $4::TEXT = 'asc' THEN b.hostname END ASC,
              CASE WHEN $3::TEXT = 'version' AND $4::TEXT = 'desc' THEN b.bridge_version END DESC,
              CASE WHEN $3::TEXT = 'version' AND $4::TEXT = 'asc' THEN b.bridge_version END ASC,
              b.session_id
            LIMIT $5 OFFSET $6"#,
        days,
        query.stale_only,
        query.sort,
        query.dir,
        query.limit,
        query.offset,
    )
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "total!"
             FROM bridge_sessions b
            WHERE NOT $2::BOOLEAN
               OR b.last_heartbeat_at < NOW() - make_interval(days => $1)"#,
        days,
        query.stale_only,
    )
    .fetch_one(pool)
    .await?;

    Ok((rows, total))
}

#[derive(Debug, Clone)]
pub struct VersionCount {
    pub bridge_version: String,
    pub devices: i64,
}

// Why: the histogram is capped because it is a shape, not an inventory. A
// fleet mid-rollout has two or three versions that matter and a long tail of
// machines nobody has touched; twelve bars show the former without the page
// growing with the latter.
pub async fn list_version_counts(pool: &PgPool) -> Result<Vec<VersionCount>, sqlx::Error> {
    sqlx::query_as!(
        VersionCount,
        r#"SELECT bridge_version AS "bridge_version!", COUNT(*) AS "devices!"
             FROM bridge_sessions
            GROUP BY bridge_version
            ORDER BY COUNT(*) DESC, bridge_version DESC
            LIMIT 12"#
    )
    .fetch_all(pool)
    .await
}
