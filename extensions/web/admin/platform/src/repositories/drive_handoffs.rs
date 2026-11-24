//! One-shot session handoff codes (`handoff`).
//!
//! Only the SHA-256 of a code is stored, bound to its person, the `/drive/`
//! path and the `ui` session's sign-in time. Storing a code sweeps the
//! expired ones, and only stores it while the `ui` session it comes from is
//! live; spending one deletes it, live or not, and answers only a live code
//! of an active person, read at that moment so a role change since the issue
//! is honoured.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

#[derive(Debug)]
pub struct NewDriveHandoff<'a> {
    pub code_hash: &'a str,
    pub user: &'a UserId,
    pub session: &'a SessionId,
    pub path: &'a str,
    pub authenticated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DriveHandoff {
    pub user_id: UserId,
    pub email: String,
    pub display_name: String,
    pub roles: Vec<String>,
    pub path: String,
    pub authenticated_at: DateTime<Utc>,
}

pub async fn insert_drive_handoff(
    pool: &PgPool,
    handoff: &NewDriveHandoff<'_>,
) -> Result<bool, sqlx::Error> {
    let stored = sqlx::query!(
        r#"WITH expired AS (
               DELETE FROM drive_handoffs WHERE expires_at <= CURRENT_TIMESTAMP
           )
           INSERT INTO drive_handoffs (code_hash, user_id, path, authenticated_at, expires_at)
           SELECT $1, $2, $4, $5, $6
            WHERE EXISTS (
                  SELECT 1
                    FROM user_sessions s
                    JOIN users u ON u.id = s.user_id
                   WHERE s.session_id = $3
                     AND s.user_id = $2
                     AND s.revoked_at IS NULL
                     AND s.expires_at > CURRENT_TIMESTAMP
                     AND u.status = 'active')"#,
        handoff.code_hash,
        handoff.user.as_str(),
        handoff.session.as_str(),
        handoff.path,
        handoff.authenticated_at,
        handoff.expires_at,
    )
    .execute(pool)
    .await?;
    Ok(stored.rows_affected() == 1)
}

pub async fn delete_drive_handoff(
    pool: &PgPool,
    code_hash: &str,
) -> Result<Option<DriveHandoff>, sqlx::Error> {
    sqlx::query_as!(
        DriveHandoff,
        r#"WITH spent AS (
               DELETE FROM drive_handoffs WHERE code_hash = $1
               RETURNING user_id, path, authenticated_at, expires_at
           )
           SELECT u.id AS "user_id!: UserId",
                  u.email AS "email!",
                  COALESCE(u.display_name, u.name) AS "display_name!",
                  u.roles AS "roles!: Vec<String>",
                  spent.path AS "path!",
                  spent.authenticated_at AS "authenticated_at!"
             FROM spent
             JOIN users u ON u.id = spent.user_id
            WHERE spent.expires_at > CURRENT_TIMESTAMP
              AND u.status = 'active'"#,
        code_hash,
    )
    .fetch_optional(pool)
    .await
}
