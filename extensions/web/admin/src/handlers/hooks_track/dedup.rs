//! Stable event identity shared by first delivery and retries.

use sha2::{Digest, Sha256};
use systemprompt::identifiers::{SessionId, UserId};
use systemprompt_web_admin_core::types::webhook::HookEventPayload;

pub(super) fn compute_dedup_key(
    user_id: &UserId,
    session_id: &SessionId,
    payload: &HookEventPayload,
) -> Result<String, String> {
    let identity = (
        user_id.as_str(),
        session_id.as_str(),
        payload.event_name(),
        payload.delivery_id()?,
    );
    let encoded = serde_json::to_vec(&identity).map_err(|e| e.to_string())?;
    Ok(hex::encode(Sha256::digest(encoded)))
}
