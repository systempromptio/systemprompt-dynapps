//! Personal access tokens, read for the whole estate rather than one account.
//!
//! Only the prefix is stored in a form that can be shown, which is the point:
//! the listing identifies a credential well enough to revoke it and not well
//! enough to use it. A row's state is derived from two nullable timestamps —
//! revoked beats expired, because a revoked token that later passes its expiry
//! is still revoked and reporting it otherwise would suggest it lapsed on its
//! own.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone)]
pub struct FleetApiKeyRow {
    pub id: String,
    pub user_id: UserId,
    pub user_name: String,
    pub name: String,
    pub key_prefix: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy)]
pub struct CredentialQuery<'a> {
    // Why: `all`, `active` or `revoked`. Matched in SQL rather than branched
    // in Rust so the count and the page come from one predicate.
    pub state: &'a str,
    pub sort: &'a str,
    pub dir: &'a str,
    pub limit: i64,
    pub offset: i64,
}

pub async fn list_api_keys_paged(
    pool: &PgPool,
    query: CredentialQuery<'_>,
) -> Result<(Vec<FleetApiKeyRow>, i64), sqlx::Error> {
    let rows = sqlx::query_as!(
        FleetApiKeyRow,
        r#"SELECT k.id AS "id!", k.user_id AS "user_id!: UserId",
                  u.name AS "user_name!", k.name AS "name!",
                  k.key_prefix AS "key_prefix!", k.created_at AS "created_at!",
                  k.last_used_at, k.expires_at, k.revoked_at
             FROM user_api_keys k
             JOIN users u ON u.id = k.user_id
            WHERE ($1::TEXT = 'all')
               OR ($1::TEXT = 'active' AND k.revoked_at IS NULL)
               OR ($1::TEXT = 'revoked' AND k.revoked_at IS NOT NULL)
            ORDER BY
              CASE WHEN $3::TEXT = 'desc' THEN
                CASE $2::TEXT WHEN 'used' THEN EXTRACT(EPOCH FROM k.last_used_at)
                        WHEN 'expires' THEN EXTRACT(EPOCH FROM k.expires_at)
                        ELSE EXTRACT(EPOCH FROM k.created_at) END
              END DESC NULLS LAST,
              CASE WHEN $3::TEXT = 'asc' THEN
                CASE $2::TEXT WHEN 'used' THEN EXTRACT(EPOCH FROM k.last_used_at)
                        WHEN 'expires' THEN EXTRACT(EPOCH FROM k.expires_at)
                        ELSE EXTRACT(EPOCH FROM k.created_at) END
              END ASC NULLS LAST,
              CASE WHEN $2::TEXT = 'name' AND $3::TEXT = 'desc' THEN k.name END DESC,
              CASE WHEN $2::TEXT = 'name' AND $3::TEXT = 'asc' THEN k.name END ASC,
              k.id
            LIMIT $4 OFFSET $5"#,
        query.state,
        query.sort,
        query.dir,
        query.limit,
        query.offset,
    )
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "total!"
             FROM user_api_keys k
            WHERE ($1::TEXT = 'all')
               OR ($1::TEXT = 'active' AND k.revoked_at IS NULL)
               OR ($1::TEXT = 'revoked' AND k.revoked_at IS NOT NULL)"#,
        query.state,
    )
    .fetch_one(pool)
    .await?;

    Ok((rows, total))
}

// Why: an admin revoking someone else's token is a different act from a user
// revoking their own, and it is a different statement: there is no `user_id`
// in the predicate. The route that reaches it is gated on the admin roles,
// which is where that authority is granted and the only place it is.
pub async fn revoke_any_api_key(pool: &PgPool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"UPDATE user_api_keys
              SET revoked_at = CURRENT_TIMESTAMP
            WHERE id = $1 AND revoked_at IS NULL"#,
        id,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
