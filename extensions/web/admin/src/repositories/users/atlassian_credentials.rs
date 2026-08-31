//! Persistence for per-user Atlassian OAuth credentials.
//!
//! The refresh token here rotates on every use, so this module is deliberately
//! narrow: the only read that hands one out is [`find_for_update`], which
//! requires a transaction and takes a row lock. There is no plain `find` — a
//! caller that could read the token without the lock would eventually spend it
//! concurrently with another and sign the user out.

use sqlx::{PgPool, Postgres, Transaction};
use systemprompt::identifiers::UserId;

use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, Clone)]
pub struct AtlassianCredentialRow {
    pub encrypted_refresh_token: Vec<u8>,
    pub refresh_token_nonce: Vec<u8>,
    pub encrypted_access_token: Option<Vec<u8>>,
    pub access_token_nonce: Option<Vec<u8>>,
    pub access_token_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub cloud_id: Option<String>,
    pub cloud_url: Option<String>,
}

/// The sealed tokens plus the expiry the caller should record for them.
#[derive(Debug, Clone)]
pub struct SealedTokens {
    pub refresh_cipher: Vec<u8>,
    pub refresh_nonce: Vec<u8>,
    pub access_cipher: Vec<u8>,
    pub access_nonce: Vec<u8>,
    pub access_expires_at: chrono::DateTime<chrono::Utc>,
}

// Why: takes the row lock. Held until the caller's transaction ends, which for
// the mint path spans the Atlassian round trip on purpose — see the comment in
// `services::atlassian_oauth::get_access_token`.
pub async fn find_for_update(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &UserId,
) -> Result<Option<AtlassianCredentialRow>, MarketplaceError> {
    let row = sqlx::query!(
        "SELECT encrypted_refresh_token, refresh_token_nonce, encrypted_access_token, \
         access_token_nonce, access_token_expires_at, cloud_id, cloud_url \
         FROM atlassian_user_credentials WHERE user_id = $1 FOR UPDATE",
        user_id.as_str(),
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(row.map(|r| AtlassianCredentialRow {
        encrypted_refresh_token: r.encrypted_refresh_token,
        refresh_token_nonce: r.refresh_token_nonce,
        encrypted_access_token: r.encrypted_access_token,
        access_token_nonce: r.access_token_nonce,
        access_token_expires_at: r.access_token_expires_at,
        cloud_id: r.cloud_id,
        cloud_url: r.cloud_url,
    }))
}

// Why: called from the OAuth callback, which is the only place a credential is
// created. An upsert rather than an insert so re-consenting replaces the old
// grant instead of erroring — a user who reconnects expects it to just work.
#[derive(Debug)]
pub struct NewCredential<'a> {
    pub user_id: &'a UserId,
    pub account_id: &'a str,
    pub tokens: &'a SealedTokens,
    pub cloud_id: Option<&'a str>,
    pub cloud_url: Option<&'a str>,
    pub granted_scopes: &'a str,
}

pub async fn upsert_credential(
    pool: &PgPool,
    new: &NewCredential<'_>,
) -> Result<(), MarketplaceError> {
    let NewCredential {
        user_id,
        account_id,
        tokens,
        cloud_id,
        cloud_url,
        granted_scopes,
    } = *new;
    sqlx::query!(
        "INSERT INTO atlassian_user_credentials (user_id, atlassian_account_id, \
         encrypted_refresh_token, refresh_token_nonce, encrypted_access_token, \
         access_token_nonce, access_token_expires_at, cloud_id, cloud_url, granted_scopes) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
         ON CONFLICT (user_id) DO UPDATE SET \
           atlassian_account_id = EXCLUDED.atlassian_account_id, \
           encrypted_refresh_token = EXCLUDED.encrypted_refresh_token, \
           refresh_token_nonce = EXCLUDED.refresh_token_nonce, \
           encrypted_access_token = EXCLUDED.encrypted_access_token, \
           access_token_nonce = EXCLUDED.access_token_nonce, \
           access_token_expires_at = EXCLUDED.access_token_expires_at, \
           cloud_id = EXCLUDED.cloud_id, \
           cloud_url = EXCLUDED.cloud_url, \
           granted_scopes = EXCLUDED.granted_scopes, \
           updated_at = NOW()",
        user_id.as_str(),
        account_id,
        tokens.refresh_cipher.as_slice(),
        tokens.refresh_nonce.as_slice(),
        tokens.access_cipher.as_slice(),
        tokens.access_nonce.as_slice(),
        tokens.access_expires_at,
        cloud_id,
        cloud_url,
        granted_scopes,
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Why: the write half of a refresh, in the same transaction that read the old
// token under lock. Committing this is what makes the rotation survive.
pub async fn store_refreshed(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &UserId,
    tokens: &SealedTokens,
) -> Result<(), MarketplaceError> {
    sqlx::query!(
        "UPDATE atlassian_user_credentials SET encrypted_refresh_token = $1, \
         refresh_token_nonce = $2, encrypted_access_token = $3, access_token_nonce = $4, \
         access_token_expires_at = $5, updated_at = NOW() WHERE user_id = $6",
        tokens.refresh_cipher.as_slice(),
        tokens.refresh_nonce.as_slice(),
        tokens.access_cipher.as_slice(),
        tokens.access_nonce.as_slice(),
        tokens.access_expires_at,
        user_id.as_str(),
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn delete_credential(pool: &PgPool, user_id: &UserId) -> Result<bool, MarketplaceError> {
    let result = sqlx::query!(
        "DELETE FROM atlassian_user_credentials WHERE user_id = $1",
        user_id.as_str(),
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

// Why: the authz dimension's only question — does this user have a credential?
// Deliberately does not read the token, so it needs no lock and no DEK.
pub async fn is_atlassian_linked(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<bool, MarketplaceError> {
    let row = sqlx::query!(
        "SELECT 1 AS linked FROM atlassian_user_credentials WHERE user_id = $1",
        user_id.as_str(),
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.is_some())
}
