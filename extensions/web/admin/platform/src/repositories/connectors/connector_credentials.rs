//! Encrypted hosted MCP grants and single-use, user-bound consent state.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use systemprompt::identifiers::UserId;

#[derive(Debug)]
pub struct EncryptedGrant {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub async fn store(
    tx: &mut Transaction<'_, Postgres>,
    user: &UserId,
    provider: &str,
    grant: &EncryptedGrant,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO mcp_connector_credentials (user_id, provider, ciphertext, nonce) \
         VALUES ($1, $2, $3, $4) ON CONFLICT (user_id, provider) DO UPDATE SET \
         ciphertext = EXCLUDED.ciphertext, nonce = EXCLUDED.nonce, updated_at = NOW()",
        user.as_str(),
        provider,
        &grant.ciphertext,
        &grant.nonce,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

#[derive(Debug)]
pub struct LockedGrant {
    pub grant: EncryptedGrant,
    pub updated_at: DateTime<Utc>,
}

pub async fn lock(
    tx: &mut Transaction<'_, Postgres>,
    user: &UserId,
    provider: &str,
) -> Result<Option<LockedGrant>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT ciphertext, nonce, updated_at FROM mcp_connector_credentials \
         WHERE user_id = $1 AND provider = $2 FOR UPDATE",
        user.as_str(),
        provider,
    )
    .fetch_optional(&mut **tx)
    .await?;
    Ok(row.map(|row| LockedGrant {
        grant: EncryptedGrant {
            ciphertext: row.ciphertext,
            nonce: row.nonce,
        },
        updated_at: row.updated_at,
    }))
}

pub async fn delete(
    tx: &mut Transaction<'_, Postgres>,
    user: &UserId,
    provider: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM mcp_connector_credentials WHERE user_id = $1 AND provider = $2",
        user.as_str(),
        provider
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn purge_expired_consent_credentials(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM mcp_connector_oauth_states WHERE expires_at <= NOW()")
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn save_state(
    pool: &PgPool,
    user: &UserId,
    provider: &str,
    state: &str,
    grant: &EncryptedGrant,
) -> Result<(), sqlx::Error> {
    purge_expired_consent_credentials(pool).await?;
    sqlx::query!(
        "INSERT INTO mcp_connector_oauth_states (state, user_id, provider, ciphertext, nonce) \
         VALUES ($1, $2, $3, $4, $5)",
        state,
        user.as_str(),
        provider,
        &grant.ciphertext,
        &grant.nonce,
    )
    .execute(pool)
    .await?;
    Ok(())
}

// Why: Postgres `DELETE … RETURNING` is atomic, so each OAuth state is
// redeemable once even across processes; a SELECT-then-DELETE would allow
// replay.
pub async fn consume_state(
    pool: &PgPool,
    user: &UserId,
    provider: &str,
    state: &str,
) -> Result<Option<EncryptedGrant>, sqlx::Error> {
    sqlx::query_as!(
        EncryptedGrant,
        "DELETE FROM mcp_connector_oauth_states \
         WHERE state = $1 AND user_id = $2 AND provider = $3 AND expires_at > NOW() \
         RETURNING ciphertext, nonce",
        state,
        user.as_str(),
        provider,
    )
    .fetch_optional(pool)
    .await
}
