//! Enrolment links issued and not yet claimed.
//!
//! A link is a ten-minute promise: a code was handed to somebody so their
//! machine could enrol itself. Only the hash is stored, so nothing here can
//! reveal the code — the listing exists to answer "who is mid-enrolment right
//! now", which is the question an operator asks when a colleague says the
//! bridge will not connect.
//!
//! Expired codes are listed alongside live ones rather than filtered away. A
//! code that ran out is the commonest reason an enrolment failed, and a page
//! that hid it would answer "nothing pending" to somebody staring at a
//! bridge that never linked.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone)]
pub struct PendingLinkRow {
    pub user_id: UserId,
    pub user_name: String,
    pub user_email: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_expired: bool,
}

pub async fn list_pending_links_paged(
    pool: &PgPool,
    sort: &str,
    dir: &str,
    limit: i64,
    offset: i64,
) -> Result<(Vec<PendingLinkRow>, i64), sqlx::Error> {
    let rows = sqlx::query_as!(
        PendingLinkRow,
        r#"SELECT c.user_id AS "user_id!: UserId", u.name AS "user_name!",
                  u.email AS "user_email!", c.created_at AS "created_at!",
                  c.expires_at AS "expires_at!",
                  (c.expires_at < NOW()) AS "is_expired!"
             FROM bridge_exchange_codes c
             JOIN users u ON u.id = c.user_id
            WHERE c.consumed_at IS NULL
            ORDER BY
              CASE WHEN $2::TEXT = 'desc' THEN
                CASE $1::TEXT WHEN 'expires' THEN EXTRACT(EPOCH FROM c.expires_at)
                        ELSE EXTRACT(EPOCH FROM c.created_at) END
              END DESC NULLS LAST,
              CASE WHEN $2::TEXT = 'asc' THEN
                CASE $1::TEXT WHEN 'expires' THEN EXTRACT(EPOCH FROM c.expires_at)
                        ELSE EXTRACT(EPOCH FROM c.created_at) END
              END ASC NULLS LAST,
              c.code_hash
            LIMIT $3 OFFSET $4"#,
        sort,
        dir,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "total!"
             FROM bridge_exchange_codes
            WHERE consumed_at IS NULL"#
    )
    .fetch_one(pool)
    .await?;

    Ok((rows, total))
}
