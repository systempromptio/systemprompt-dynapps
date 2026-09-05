//! Storage for the AD groups a user held at their most recent ADFS sign-in.
//!
//! The ADFS callback writes the *mapped* groups here (the ones `adfs.yaml`
//! knows about) so the `adfs_group` subject dimension can bind entity rules
//! to them. Lives in the web-owned `adfs_group_memberships` side table
//! (schema/15), not the vendored `federated_identities` table.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

// Why: Replace, not union. The token is the whole truth about the user's
// current membership, so a group they have left must disappear here too —
// otherwise leaving an AD group would never revoke anything.
pub async fn replace_groups(
    pool: &PgPool,
    user_id: &UserId,
    groups: &[String],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query!(
        "DELETE FROM adfs_group_memberships WHERE user_id = $1",
        user_id.as_str()
    )
    .execute(&mut *tx)
    .await?;
    for group in groups {
        sqlx::query!(
            "INSERT INTO adfs_group_memberships (user_id, group_name) VALUES ($1, $2) \
             ON CONFLICT (user_id, group_name) DO UPDATE SET updated_at = CURRENT_TIMESTAMP",
            user_id.as_str(),
            group
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await
}

// Why: The groups recorded for `user_id`, sorted so callers can compare and
// display without caring about insertion order.
pub async fn list_groups(pool: &PgPool, user_id: &UserId) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT group_name FROM adfs_group_memberships WHERE user_id = $1 ORDER BY group_name",
        user_id.as_str()
    )
    .fetch_all(pool)
    .await
}
