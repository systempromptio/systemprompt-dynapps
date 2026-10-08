//! Group membership reads shared across admin domains.
//!
//! The group ids a user currently holds, which the group subject-attribute
//! provider, the user detail page and the connection entitlement resolve
//! against, and the AD group names the directory asserted at sign-in, which
//! the bridge identity card and the user detail page show. Writes stay with
//! the groups repository that owns the table and its two writers.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_shared::GroupId;

pub async fn list_group_ids_for_user(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<GroupId>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT group_id AS "group_id!: GroupId" FROM user_groups WHERE user_id = $1 ORDER BY group_id"#,
        user_id.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn list_source_ad_groups(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT DISTINCT source_ad_group AS "source_ad_group!" FROM group_members
           WHERE user_id = $1 AND source = 'adfs' AND source_ad_group IS NOT NULL
           ORDER BY 1"#,
        user_id.as_str()
    )
    .fetch_all(pool)
    .await
}
