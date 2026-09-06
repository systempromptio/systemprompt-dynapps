//! Device certificates: the long-lived key a bridge installation presents.
//!
//! A certificate outlives the session it was enrolled for, so revoking one is
//! the only act that actually removes a machine from the estate — a bridge
//! session merely stops beating. The fingerprint is shown in full because it
//! is the identifier an operator matches against the machine in front of them,
//! and it is not a secret.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::pats::CredentialQuery;

#[derive(Debug, Clone)]
pub struct FleetCertRow {
    pub id: String,
    pub user_id: UserId,
    pub user_name: String,
    pub label: String,
    pub fingerprint: String,
    pub enrolled_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

pub async fn list_device_certs_paged(
    pool: &PgPool,
    query: CredentialQuery<'_>,
) -> Result<(Vec<FleetCertRow>, i64), sqlx::Error> {
    let rows = sqlx::query_as!(
        FleetCertRow,
        r#"SELECT c.id AS "id!", c.user_id AS "user_id!: UserId",
                  u.name AS "user_name!", c.label AS "label!",
                  c.fingerprint AS "fingerprint!", c.enrolled_at AS "enrolled_at!",
                  c.revoked_at
             FROM user_device_certs c
             JOIN users u ON u.id = c.user_id
            WHERE ($1::TEXT = 'all')
               OR ($1::TEXT = 'active' AND c.revoked_at IS NULL)
               OR ($1::TEXT = 'revoked' AND c.revoked_at IS NOT NULL)
            ORDER BY
              CASE WHEN $3::TEXT = 'desc' THEN EXTRACT(EPOCH FROM c.enrolled_at) END
                DESC NULLS LAST,
              CASE WHEN $3::TEXT = 'asc' THEN EXTRACT(EPOCH FROM c.enrolled_at) END
                ASC NULLS LAST,
              CASE WHEN $2::TEXT = 'label' AND $3::TEXT = 'desc' THEN c.label END DESC,
              CASE WHEN $2::TEXT = 'label' AND $3::TEXT = 'asc' THEN c.label END ASC,
              c.id
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
             FROM user_device_certs c
            WHERE ($1::TEXT = 'all')
               OR ($1::TEXT = 'active' AND c.revoked_at IS NULL)
               OR ($1::TEXT = 'revoked' AND c.revoked_at IS NOT NULL)"#,
        query.state,
    )
    .fetch_one(pool)
    .await?;

    Ok((rows, total))
}

pub async fn revoke_any_device_cert(pool: &PgPool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"UPDATE user_device_certs
              SET revoked_at = CURRENT_TIMESTAMP
            WHERE id = $1 AND revoked_at IS NULL"#,
        id,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
