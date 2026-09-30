//! The two halves of a user's role set, and the recomputation that joins
//! them.
//!
//! `users.roles` is the effective set every authorisation check reads, but it
//! has two independent writers: the directory projects roles from the AD
//! groups an assertion carried, and an admin grants roles by hand in the
//! dashboard. Only the manual half is stored — `user_manual_roles` — so the
//! directory half is whatever the effective set holds beyond it. That keeps
//! one row per manual grant and no second copy of what AD already says.
//!
//! A manual grant carries a validity window. The reads that decide what a
//! person holds see only rows inside it; the provenance read
//! (`directory_roles_on`) sees every row, expired included, because an expired
//! manual grant is still not something the directory said. The hourly sweep
//! deletes expired rows and rewrites `users.roles` from what is left
//! (`expire_manual_roles`).
//!
//! The directory half is derived, never stored: the directory rewrites
//! `users.roles` at every sign-in, so a stored copy could only go stale. Every
//! write that removes manual rows snapshots the directory half first, inside
//! the same transaction — otherwise "effective minus manual" would absorb the
//! removed grant and keep it alive.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use systemprompt_web_admin_core::types::constants::ROLE_PLATFORM_ADMIN;

pub async fn list_manual_roles(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT role FROM user_manual_roles
         WHERE user_id = $1 AND valid_from <= CURRENT_TIMESTAMP
           AND (valid_until IS NULL OR valid_until > CURRENT_TIMESTAMP)
         ORDER BY role",
        user_id.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn find_manual_roles_valid_until(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT MIN(valid_until) AS "valid_until?" FROM user_manual_roles WHERE user_id = $1"#,
        user_id.as_str()
    )
    .fetch_one(pool)
    .await
}

pub async fn list_directory_roles(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<String>, sqlx::Error> {
    directory_roles_on(pool, user_id).await
}

async fn directory_roles_on<'e, E>(
    executor: E,
    user_id: &UserId,
) -> Result<Vec<String>, sqlx::Error>
where
    E: sqlx::PgExecutor<'e>,
{
    sqlx::query_scalar!(
        r#"
        SELECT r AS "r!" FROM users u, UNNEST(u.roles) AS r
        WHERE u.id = $1
          AND NOT EXISTS (
              SELECT 1 FROM user_manual_roles m
              WHERE m.user_id = u.id AND m.role = r)
        ORDER BY r
        "#,
        user_id.as_str()
    )
    .fetch_all(executor)
    .await
}

pub async fn count_platform_admins(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT COUNT(*)::BIGINT AS "count!" FROM users WHERE $1 = ANY(roles)"#,
        ROLE_PLATFORM_ADMIN
    )
    .fetch_one(pool)
    .await
}

pub async fn set_manual_roles(
    pool: &PgPool,
    user_id: &UserId,
    roles: &[String],
    granted_by: &UserId,
    valid_until: Option<DateTime<Utc>>,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    let directory = directory_roles_on(&mut *tx, user_id).await?;
    sqlx::query!(
        "DELETE FROM user_manual_roles WHERE user_id = $1",
        user_id.as_str()
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "INSERT INTO user_manual_roles (user_id, role, granted_by, valid_until)
         SELECT $1, r, $3, $4 FROM UNNEST($2::TEXT[]) AS r ON CONFLICT DO NOTHING",
        user_id.as_str(),
        roles,
        granted_by.as_str(),
        valid_until
    )
    .execute(&mut *tx)
    .await?;
    write_effective_roles(&mut *tx, user_id, &directory).await?;
    tx.commit().await
}

pub async fn recompute_roles(
    pool: &PgPool,
    user_id: &UserId,
    directory_roles: Option<&[String]>,
) -> Result<Vec<String>, sqlx::Error> {
    let directory = match directory_roles {
        Some(roles) => roles.to_vec(),
        None => list_directory_roles(pool, user_id).await?,
    };
    write_effective_roles(pool, user_id, &directory).await
}

async fn write_effective_roles<'e, E>(
    executor: E,
    user_id: &UserId,
    directory: &[String],
) -> Result<Vec<String>, sqlx::Error>
where
    E: sqlx::PgExecutor<'e>,
{
    sqlx::query_scalar!(
        r#"
        UPDATE users SET roles = (
            SELECT COALESCE(ARRAY_AGG(DISTINCT r ORDER BY r), ARRAY[]::TEXT[])
            FROM (
                SELECT UNNEST($2::TEXT[]) AS r
                UNION
                SELECT role FROM user_manual_roles
                 WHERE user_id = $1 AND valid_from <= CURRENT_TIMESTAMP
                   AND (valid_until IS NULL OR valid_until > CURRENT_TIMESTAMP)
            ) all_roles
        )
        WHERE id = $1
        RETURNING roles AS "roles!"
        "#,
        user_id.as_str(),
        directory
    )
    .fetch_one(executor)
    .await
}

pub async fn expire_manual_roles(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<(Vec<String>, Vec<String>), sqlx::Error> {
    let mut tx = pool.begin().await?;
    let before = sqlx::query_scalar!(
        r#"SELECT roles AS "roles!: Vec<String>" FROM users WHERE id = $1 FOR UPDATE"#,
        user_id.as_str()
    )
    .fetch_one(&mut *tx)
    .await?;
    let directory = directory_roles_on(&mut *tx, user_id).await?;
    sqlx::query!(
        "DELETE FROM user_manual_roles
         WHERE user_id = $1 AND valid_until IS NOT NULL AND valid_until <= CURRENT_TIMESTAMP",
        user_id.as_str()
    )
    .execute(&mut *tx)
    .await?;
    let after = write_effective_roles(&mut *tx, user_id, &directory).await?;
    tx.commit().await?;
    Ok((before, after))
}

pub async fn list_users_with_expired_manual_roles(
    pool: &PgPool,
) -> Result<Vec<UserId>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT DISTINCT user_id AS "user_id!: UserId" FROM user_manual_roles
           WHERE valid_until IS NOT NULL AND valid_until <= CURRENT_TIMESTAMP
           ORDER BY user_id"#
    )
    .fetch_all(pool)
    .await
}
