//! Sealed machine credentials held by service accounts, one per provider.
//!
//! Reads that feed the console never select the sealed secret; only the
//! broker's `find_machine_credential` returns it.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

// JSON: `config` holds each kind's non-secret identifiers (App and
// installation ids, instance URL and consumer key, token owner email); the
// keys differ per kind, so the column is JSONB and stays untyped here.
#[derive(Debug)]
pub struct NewMachineCredential<'a> {
    pub user: &'a UserId,
    pub provider: &'a str,
    pub kind: &'a str,
    pub config: serde_json::Value,
    pub secret_ciphertext: Vec<u8>,
    pub secret_nonce: Vec<u8>,
    pub created_by: &'a str,
}

#[derive(Debug)]
pub struct MachineCredentialRow {
    pub kind: String,
    pub config: serde_json::Value,
    pub secret_ciphertext: Vec<u8>,
    pub secret_nonce: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
}

// JSON: the same per-kind identifiers, read back for display.
#[derive(Debug, Clone)]
pub struct MachineCredentialSummary {
    pub user_id: UserId,
    pub provider: String,
    pub kind: String,
    pub config: serde_json::Value,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
}

// Why: Postgres `ON CONFLICT` makes attach-again a rotation in one statement:
// it keeps `created_by`/`created_at` and stamps `rotated_at`.
pub async fn set_machine_credential(
    pool: &PgPool,
    credential: &NewMachineCredential<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO mcp_connector_machine_credentials \
         (user_id, provider, kind, config, secret_ciphertext, secret_nonce, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (user_id, provider) DO UPDATE SET \
         kind = EXCLUDED.kind, config = EXCLUDED.config, \
         secret_ciphertext = EXCLUDED.secret_ciphertext, \
         secret_nonce = EXCLUDED.secret_nonce, rotated_at = NOW()",
        credential.user.as_str(),
        credential.provider,
        credential.kind,
        credential.config,
        &credential.secret_ciphertext,
        &credential.secret_nonce,
        credential.created_by,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find_machine_credential(
    pool: &PgPool,
    user: &UserId,
    provider: &str,
) -> Result<Option<MachineCredentialRow>, sqlx::Error> {
    sqlx::query_as!(
        MachineCredentialRow,
        "SELECT kind, config, secret_ciphertext, secret_nonce, created_at, rotated_at \
         FROM mcp_connector_machine_credentials WHERE user_id = $1 AND provider = $2",
        user.as_str(),
        provider,
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_machine_credentials(
    pool: &PgPool,
) -> Result<Vec<MachineCredentialSummary>, sqlx::Error> {
    sqlx::query_as!(
        MachineCredentialSummary,
        "SELECT user_id AS \"user_id: UserId\", provider, kind, config, created_by, created_at, \
         rotated_at FROM mcp_connector_machine_credentials ORDER BY user_id, provider",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_machine_credentials_for_user(
    pool: &PgPool,
    user: &UserId,
) -> Result<Vec<MachineCredentialSummary>, sqlx::Error> {
    sqlx::query_as!(
        MachineCredentialSummary,
        "SELECT user_id AS \"user_id: UserId\", provider, kind, config, created_by, created_at, \
         rotated_at FROM mcp_connector_machine_credentials WHERE user_id = $1 ORDER BY provider",
        user.as_str(),
    )
    .fetch_all(pool)
    .await
}

pub async fn count_machine_credentials(pool: &PgPool, user: &UserId) -> Result<i64, sqlx::Error> {
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) AS \"count!\" FROM mcp_connector_machine_credentials WHERE user_id = $1",
        user.as_str(),
    )
    .fetch_one(pool)
    .await?;
    Ok(count)
}

pub async fn delete_machine_credential(
    pool: &PgPool,
    user: &UserId,
    provider: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM mcp_connector_machine_credentials WHERE user_id = $1 AND provider = $2",
        user.as_str(),
        provider,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
