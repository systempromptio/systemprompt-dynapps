//! Hook token validation for `/hooks/track`.
//!
//! Hook JWTs carry `aud=hook` and scope `hook:track`, distinct from the API and
//! plugin audiences.

use crate::error::{AdminError, AdminResult};
use axum::http::HeaderMap;
use systemprompt::identifiers::{PluginId, UserId};
use systemprompt::models::Config;
use systemprompt_security::HookTokenValidator;

pub(super) fn extract_and_validate_jwt(
    headers: &HeaderMap,
) -> AdminResult<(UserId, PluginId, String)> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AdminError::Unauthorized("Missing Authorization header".to_owned()))?;
    let jwt_issuer = Config::get()?.jwt_issuer.clone();
    // Why: `None` skips the request-vs-claim plugin_id cross-check — this
    // endpoint takes no plugin_id path/query binding to compare against.
    let claims = HookTokenValidator::new(jwt_issuer)
        .validate_track(token, None)
        .map_err(AdminError::unauthenticated)?;
    Ok((claims.subject, claims.plugin_id, token.to_owned()))
}

pub(super) async fn authenticated_device(
    pool: &sqlx::PgPool,
    headers: &HeaderMap,
    user: &UserId,
) -> AdminResult<Option<crate::repositories::marketplace::webhook::AuthenticatedHookDevice>> {
    let Some(credential) = headers.get("x-systemprompt-device-credential") else {
        return Ok(None);
    };
    let credential = credential
        .to_str()
        .map_err(|_| AdminError::Unauthorized("Invalid device credential".to_owned()))?;
    let host: systemprompt::models::feedback::EvaluatorClient =
        serde_json::from_value(serde_json::Value::String(
            headers
                .get("x-systemprompt-host")
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| AdminError::BadRequest("Missing device host".to_owned()))?
                .to_owned(),
        ))
        .map_err(|_| AdminError::BadRequest("Invalid device host".to_owned()))?;
    let identity = systemprompt::marketplace::managed::ManagedRepository::new(pool.clone())
        .authenticate_consumer_device(credential)
        .await
        .map_err(|_| {
            AdminError::Unauthorized("Device credential is unavailable or revoked".to_owned())
        })?;
    if &identity.consumer_id != user {
        return Err(AdminError::Forbidden(
            "Hook consumer does not match enrolled device".to_owned(),
        ));
    }
    Ok(Some(
        crate::repositories::marketplace::webhook::AuthenticatedHookDevice {
            consumer_id: identity.consumer_id,
            device_id: identity.device_id,
            host,
        },
    ))
}
