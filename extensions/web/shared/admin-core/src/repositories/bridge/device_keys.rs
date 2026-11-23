//! The device keys core's `session-pat` mints for a device link.
//!
//! Each link mints a new key, and core's key revocation route is for admins
//! only, so the device-link page retires the key the device says it replaces.
//! Only a live `device` key of the approving person matches; any other id
//! changes nothing, so a stale or foreign id is not an error.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::error::Result;

pub async fn revoke_device_key(pool: &PgPool, user_id: &UserId, id: &str) -> Result<bool> {
    let result = sqlx::query!(
        r#"
        UPDATE user_api_keys
        SET revoked_at = CURRENT_TIMESTAMP
        WHERE id = $1 AND user_id = $2 AND kind = 'device' AND revoked_at IS NULL
        "#,
        id,
        user_id.as_str(),
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
