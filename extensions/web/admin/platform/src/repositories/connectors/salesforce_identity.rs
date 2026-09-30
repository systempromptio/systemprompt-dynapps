//! Storage for a local user's Salesforce *Username*, one per org.
//!
//! The Salesforce JWT-bearer grant matches its `sub` claim against the
//! Salesforce Username, which is not the login email (e.g.
//! `ed.aa…@agentforce.com` vs `ed@systemprompt.io`) and differs from org to
//! org. `provider` is the MCP server id fronting the org (`salesforce`,
//! `salesforce-<slug>`), so the row is keyed `(user_id, provider)`.
//!
//! Lives in the web-owned `salesforce_user_identities` side table (schema/21),
//! not the vendored `federated_identities` table.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

pub async fn upsert_identity(
    pool: &PgPool,
    user_id: &UserId,
    provider: &str,
    sf_username: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO salesforce_user_identities (user_id, provider, sf_username) \
         VALUES ($1, $2, $3) \
         ON CONFLICT (user_id, provider) DO UPDATE \
         SET sf_username = EXCLUDED.sf_username, updated_at = CURRENT_TIMESTAMP",
        user_id.as_str(),
        provider,
        sf_username
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_identity(
    pool: &PgPool,
    user_id: &UserId,
    provider: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM salesforce_user_identities WHERE user_id = $1 AND provider = $2",
        user_id.as_str(),
        provider
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_all_identities(pool: &PgPool, user_id: &UserId) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM salesforce_user_identities WHERE user_id = $1",
        user_id.as_str()
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find_username(
    pool: &PgPool,
    user_id: &UserId,
    provider: &str,
) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT sf_username FROM salesforce_user_identities \
         WHERE user_id = $1 AND provider = $2",
        user_id.as_str(),
        provider
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.sf_username))
}

pub async fn list_linked_providers(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT provider FROM salesforce_user_identities WHERE user_id = $1 ORDER BY provider",
        user_id.as_str()
    )
    .fetch_all(pool)
    .await
}
