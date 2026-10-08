//! User create, update, archive and restore.
//!
//! Neither door touches group or project membership here: the create handler
//! writes the drawer's picks as manual grants through the membership
//! repositories, and directory-owned memberships are what the assertion says
//! at SSO sign-in (see [`super::federated`]). Roles are left unchanged by an
//! update; only the role route writes them.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_shared::error::MarketplaceError;

use systemprompt_web_admin_core::types::{CreateUserRequest, UpdateUserRequest, UserSummary};

pub async fn create_user(
    pool: &PgPool,
    req: &CreateUserRequest,
) -> Result<UserSummary, MarketplaceError> {
    let user_id_str = req.user_id.as_str().to_owned();
    let status = req.status.clone().unwrap_or_else(|| "active".to_owned());
    let username = req.email.as_str();
    let summary = sqlx::query_as!(
        UserSummary,
        r#"
        INSERT INTO users (id, name, email, display_name, roles, status)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (email) DO UPDATE SET
            display_name = COALESCE(EXCLUDED.display_name, users.display_name),
            roles = EXCLUDED.roles,
            status = EXCLUDED.status,
            updated_at = NOW()
        RETURNING
            id AS "user_id!: UserId",
            COALESCE(display_name, name) AS display_name,
            email AS "email: _",
            roles AS "roles!: Vec<String>",
            (status = 'active') AS "is_active!",
            created_at AS "last_active!",
            0::BIGINT AS "total_events!",
            NULL::TEXT AS last_tool,
            0::BIGINT AS "custom_skills_count!",
            NULL::TEXT AS preferred_client,
            0::BIGINT AS "prompts!",
            0::BIGINT AS "sessions!",
            0::BIGINT AS "bytes!",
            0::BIGINT AS "logins!"
        "#,
        &user_id_str,
        username,
        req.email.as_str(),
        &req.display_name,
        &req.roles,
        &status,
    )
    .fetch_one(pool)
    .await?;
    Ok(summary)
}

pub async fn update_user(
    pool: &PgPool,
    user_id: &UserId,
    req: &UpdateUserRequest,
) -> Result<Option<UserSummary>, sqlx::Error> {
    let status = req.is_active.map(|active| {
        if active {
            "active".to_owned()
        } else {
            "inactive".to_owned()
        }
    });
    let set_email_verified = req.is_active == Some(true);
    let unchanged_roles: Option<&[String]> = None;

    sqlx::query_as!(
        UserSummary,
        r#"
        UPDATE users
        SET
            display_name = COALESCE($2, display_name),
            email = COALESCE($3, email),
            roles = COALESCE($4, roles),
            status = COALESCE($5, status),
            email_verified = CASE WHEN $6 THEN true ELSE email_verified END,
            updated_at = NOW()
        WHERE id = $1
        RETURNING
            id AS "user_id!: UserId",
            COALESCE(display_name, name) AS display_name,
            email AS "email: _",
            roles AS "roles!: Vec<String>",
            (status = 'active') AS "is_active!",
            updated_at AS "last_active!",
            0::BIGINT AS "total_events!",
            NULL::TEXT AS last_tool,
            0::BIGINT AS "custom_skills_count!",
            NULL::TEXT AS preferred_client,
            0::BIGINT AS "prompts!",
            0::BIGINT AS "sessions!",
            0::BIGINT AS "bytes!",
            0::BIGINT AS "logins!"
        "#,
        user_id.as_str(),
        req.display_name.as_deref(),
        req.email.as_deref(),
        unchanged_roles,
        status.as_deref(),
        set_email_verified,
    )
    .fetch_optional(pool)
    .await
}

pub async fn archive_user(
    pool: &PgPool,
    user_id: &UserId,
    archived_by: &UserId,
    reason: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "UPDATE users
         SET status = 'deleted', archived_at = COALESCE(archived_at, NOW()),
             archived_by = COALESCE(archived_by, $2), archive_reason = COALESCE(archive_reason, $3),
             updated_at = NOW()
         WHERE id = $1",
        user_id.as_str(),
        archived_by.as_str(),
        reason,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn restore_user(
    pool: &PgPool,
    user_id: &UserId,
    window_days: i32,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "UPDATE users
         SET status = 'active', archived_at = NULL, archived_by = NULL, archive_reason = NULL,
             updated_at = NOW()
         WHERE id = $1 AND status = 'deleted'
           AND (archived_at IS NULL OR archived_at > NOW() - make_interval(days => $2))",
        user_id.as_str(),
        window_days,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
