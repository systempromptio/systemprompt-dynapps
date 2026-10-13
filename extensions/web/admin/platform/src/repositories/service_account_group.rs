//! Membership of the `service-accounts` group, which carries a service
//! account's entitlement to the write-back connectors in `rules.yaml`.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

pub const SERVICE_ACCOUNTS_GROUP: &str = "service-accounts";

// Why: boot seeds groups from `services/web/config/groups.yaml` only into an
// empty database, so an installation that predates the group would refuse the
// membership row; the group is upserted exactly as declared there.
pub async fn insert_service_account_membership(
    pool: &PgPool,
    user: &UserId,
    granted_by: &UserId,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query!(
        "INSERT INTO groups (id, name, description, source) VALUES ($1, 'Service accounts', \
         'Non-interactive accounts workflow write-backs act as.', 'yaml') \
         ON CONFLICT (id) DO NOTHING",
        SERVICE_ACCOUNTS_GROUP,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "INSERT INTO group_members (group_id, user_id, source, granted_by) \
         VALUES ($1, $2, 'manual', $3) ON CONFLICT (group_id, user_id, source) DO NOTHING",
        SERVICE_ACCOUNTS_GROUP,
        user.as_str(),
        granted_by.as_str(),
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}
