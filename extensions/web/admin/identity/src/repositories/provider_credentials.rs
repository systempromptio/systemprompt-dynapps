//! Who holds a personal-subscription key, read from core's
//! `user_provider_credentials` store.
//!
//! Metadata only: the sealed key (`ciphertext`, `nonce`) is never selected,
//! so no row this module returns can carry a key to a page.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone, Serialize)]
pub struct ProviderCredentialHolder {
    pub user_id: UserId,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub provider: String,
    pub label: Option<String>,
    pub created_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
}

pub async fn list_provider_credential_holders(
    pool: &PgPool,
) -> Result<Vec<ProviderCredentialHolder>, sqlx::Error> {
    sqlx::query_as!(
        ProviderCredentialHolder,
        r#"SELECT
             c.user_id AS "user_id!: UserId",
             u.email AS "email?",
             u.display_name AS "display_name?",
             c.provider,
             c.label,
             c.created_at,
             c.verified_at,
             c.last_used_at
           FROM user_provider_credentials c
           LEFT JOIN users u ON u.id = c.user_id
           ORDER BY u.email NULLS LAST, c.provider"#,
    )
    .fetch_all(pool)
    .await
}
