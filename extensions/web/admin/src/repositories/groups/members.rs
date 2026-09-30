//! Group membership, which has two writers that must not overwrite each
//! other.
//!
//! The directory replaces every `adfs` row at each sign-in — leaving an AD
//! group has to actually remove the membership — while `manual` rows an admin
//! added survive that replace. The primary key carries `source`, so both can
//! hold the same pair and a member row reports the set of sources behind it.
//!
//! Every read here is windowed: a row outside its `valid_from`/`valid_until`
//! or stamped `revoked_at` is not a membership. The `user_groups` view carries
//! the same predicate, so the two never disagree about who is in a group.
//! Re-adding a manual row the sweep revoked revives it rather than refusing.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_shared::GroupId;

use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::types::groups::GroupMemberRow;

pub async fn list_group_members(
    pool: &PgPool,
    group_id: &GroupId,
) -> Result<Vec<GroupMemberRow>, sqlx::Error> {
    sqlx::query_as!(
        GroupMemberRow,
        r#"
        SELECT
            ug.user_id AS "user_id!: UserId",
            u.display_name AS "display_name?",
            u.email AS "email?",
            COALESCE(
                (SELECT ARRAY_AGG(DISTINCT gm.source) FROM group_members gm
                 WHERE gm.group_id = ug.group_id AND gm.user_id = ug.user_id
                   AND gm.revoked_at IS NULL AND gm.valid_from <= CURRENT_TIMESTAMP
                   AND (gm.valid_until IS NULL OR gm.valid_until > CURRENT_TIMESTAMP)),
                ARRAY['derived']::TEXT[]
            ) AS "sources!",
            COALESCE(
                (SELECT ARRAY_AGG(DISTINCT gm.source_ad_group) FROM group_members gm
                 WHERE gm.group_id = ug.group_id AND gm.user_id = ug.user_id
                   AND gm.source_ad_group IS NOT NULL
                   AND gm.revoked_at IS NULL AND gm.valid_from <= CURRENT_TIMESTAMP
                   AND (gm.valid_until IS NULL OR gm.valid_until > CURRENT_TIMESTAMP)),
                ARRAY[]::TEXT[]
            ) AS "source_ad_groups!",
            (SELECT gm.valid_until FROM group_members gm
              WHERE gm.group_id = ug.group_id AND gm.user_id = ug.user_id
                AND gm.source = 'manual' AND gm.revoked_at IS NULL) AS "valid_until?"
        FROM user_groups ug
        JOIN users u ON u.id = ug.user_id
        WHERE ug.group_id = $1
        ORDER BY u.display_name NULLS LAST, ug.user_id
        "#,
        group_id.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn list_unassigned_users(pool: &PgPool) -> Result<Vec<UserId>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT user_id AS "user_id!: UserId" FROM user_groups
           WHERE group_id = 'unassigned' ORDER BY user_id"#
    )
    .fetch_all(pool)
    .await
}

pub async fn count_group_members(pool: &PgPool, group_id: &GroupId) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT COUNT(DISTINCT user_id)::BIGINT AS "count!"
           FROM user_groups WHERE group_id = $1"#,
        group_id.as_str()
    )
    .fetch_one(pool)
    .await
}

pub async fn insert_group_member(
    pool: &PgPool,
    group_id: &GroupId,
    user_id: &UserId,
    granted_by: &UserId,
    valid_until: Option<DateTime<Utc>>,
) -> AdminResult<()> {
    let inserted = sqlx::query!(
        "INSERT INTO group_members (group_id, user_id, source, granted_by, valid_until)
         VALUES ($1, $2, 'manual', $3, $4)
         ON CONFLICT (group_id, user_id, source) DO UPDATE
            SET granted_by = EXCLUDED.granted_by,
                valid_from = CURRENT_TIMESTAMP,
                valid_until = EXCLUDED.valid_until,
                revoked_at = NULL
          WHERE group_members.revoked_at IS NOT NULL",
        group_id.as_str(),
        user_id.as_str(),
        granted_by.as_str(),
        valid_until
    )
    .execute(pool)
    .await?;
    if inserted.rows_affected() == 0 {
        return Err(AdminError::Conflict(format!(
            "User {user_id} is already a manual member of {group_id}"
        )));
    }
    Ok(())
}

// Why: AD re-asserts a directory membership at the member's next sign-in, so
// deleting one here would be undone; the change belongs in AD.
pub async fn delete_group_member(
    pool: &PgPool,
    group_id: &GroupId,
    user_id: &UserId,
) -> AdminResult<()> {
    let sources = sqlx::query_scalar!(
        r#"SELECT source AS "source!" FROM group_members
           WHERE group_id = $1 AND user_id = $2 AND revoked_at IS NULL"#,
        group_id.as_str(),
        user_id.as_str()
    )
    .fetch_all(pool)
    .await?;

    if sources.is_empty() {
        return Err(AdminError::NotFound(format!(
            "User {user_id} is not a member of {group_id}"
        )));
    }
    if !sources.iter().any(|s| s == "manual") {
        return Err(AdminError::Conflict(format!(
            "User {user_id} is in {group_id} through the directory; remove them from the AD group"
        )));
    }
    sqlx::query!(
        "DELETE FROM group_members WHERE group_id = $1 AND user_id = $2 AND source = 'manual'",
        group_id.as_str(),
        user_id.as_str()
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn replace_directory_group_memberships(
    pool: &PgPool,
    user_id: &UserId,
    ad_groups: &[String],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query!(
        "DELETE FROM group_members WHERE user_id = $1 AND source = 'adfs'",
        user_id.as_str()
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "INSERT INTO group_members (group_id, user_id, source, source_ad_group)
         SELECT m.group_id, $1, 'adfs', m.ad_group FROM group_ad_mappings m
         WHERE m.ad_group = ANY($2) ON CONFLICT DO NOTHING",
        user_id.as_str(),
        ad_groups
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    systemprompt_web_admin_core::repositories::scope::defaults::recompute_scope_defaults_for_user(
        pool, user_id,
    )
    .await?;
    Ok(())
}
