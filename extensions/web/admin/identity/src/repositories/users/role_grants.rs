//! One time-boxed manual role, granted and revoked on its own.
//!
//! `set_manual_roles` replaces a user's whole manual set, which is right for
//! the role editor and wrong for an automated grant: a demo that elevates a
//! service account for half an hour must not erase what an admin granted it
//! by hand. These two writes touch one `(user, role)` row and recompute the
//! effective set in the same transaction, like every other manual-role write.
//!
//! A standing grant (no `valid_until`) wins: granting over it changes
//! nothing and answers [`TimedGrant::Standing`], and revoking only ever
//! deletes a timed row, so a demo cannot shorten or remove access somebody
//! granted permanently.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::roles::{directory_roles_on, write_effective_roles};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimedGrant {
    Granted,
    Standing,
}

pub async fn set_timed_manual_role(
    pool: &PgPool,
    user_id: &UserId,
    role: &str,
    granted_by: &UserId,
    valid_until: DateTime<Utc>,
) -> Result<TimedGrant, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let directory = directory_roles_on(&mut *tx, user_id).await?;
    let written = sqlx::query_scalar!(
        r#"INSERT INTO user_manual_roles (user_id, role, granted_by, valid_until)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (user_id, role) DO UPDATE
              SET granted_by = EXCLUDED.granted_by,
                  valid_from = CURRENT_TIMESTAMP,
                  valid_until = EXCLUDED.valid_until
            WHERE user_manual_roles.valid_until IS NOT NULL
           RETURNING role AS "role!""#,
        user_id.as_str(),
        role,
        granted_by.as_str(),
        valid_until
    )
    .fetch_optional(&mut *tx)
    .await?;
    write_effective_roles(&mut *tx, user_id, &directory).await?;
    tx.commit().await?;
    Ok(if written.is_some() {
        TimedGrant::Granted
    } else {
        TimedGrant::Standing
    })
}

pub async fn delete_timed_manual_role(
    pool: &PgPool,
    user_id: &UserId,
    role: &str,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let directory = directory_roles_on(&mut *tx, user_id).await?;
    let deleted = sqlx::query!(
        "DELETE FROM user_manual_roles
         WHERE user_id = $1 AND role = $2 AND valid_until IS NOT NULL",
        user_id.as_str(),
        role
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();
    write_effective_roles(&mut *tx, user_id, &directory).await?;
    tx.commit().await?;
    Ok(deleted > 0)
}
