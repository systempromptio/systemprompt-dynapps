//! The device keys core's `session-pat` mints for a device link.
//!
//! Each link mints a new key, and core's key revocation route is for admins
//! only, so the device-link page retires the key the device says it replaces.
//! The device names it by its `key_prefix` (`sp-live-<12 hex>`, the part of
//! the PAT before the dot, not secret), since core never tells it the key id.
//! Only an unrevoked `device` key of the approving person matches, as core's
//! `find_active_api_key_by_prefix` reads active; any other prefix changes
//! nothing, so a stale or foreign one is not an error.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::error::Result;

pub async fn revoke_device_key(pool: &PgPool, user_id: &UserId, key_prefix: &str) -> Result<bool> {
    let result = sqlx::query!(
        r#"
        UPDATE user_api_keys
        SET revoked_at = CURRENT_TIMESTAMP
        WHERE key_prefix = $1 AND user_id = $2 AND kind = 'device' AND revoked_at IS NULL
        "#,
        key_prefix,
        user_id.as_str(),
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
