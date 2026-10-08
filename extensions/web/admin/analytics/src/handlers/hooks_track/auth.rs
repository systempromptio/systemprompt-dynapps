//! Hook token validation for `/hooks/track`.
//!
//! Hook JWTs carry `aud=hook` and scope `hook:track`, distinct from the API and
//! plugin audiences.

use axum::http::HeaderMap;
use systemprompt::identifiers::{PluginId, UserId};
use systemprompt::manifest::Config;
use systemprompt_security::HookTokenValidator;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

pub(super) fn extract_and_validate_jwt(headers: &HeaderMap) -> AdminResult<(UserId, PluginId)> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AdminError::Unauthorized("Missing Authorization header".to_owned()))?;
    let jwt_issuer = Config::get()?.jwt_issuer.clone();
    let claims = HookTokenValidator::new(jwt_issuer)
        .validate_track(token, None)
        .map_err(AdminError::unauthenticated)?;
    Ok((claims.subject, claims.plugin_id))
}

pub(super) async fn authenticated_device(
    repository: &systemprompt::marketplace::managed::ManagedRepository,
    headers: &HeaderMap,
    user: &UserId,
) -> AdminResult<
    Option<systemprompt_web_admin_core::repositories::ingestion::webhook::AuthenticatedHookDevice>,
> {
    let Some(credential) = headers.get("x-systemprompt-device-credential") else {
        return Ok(None);
    };
    let credential = credential.to_str().map_err(|error| {
        tracing::debug!(%error, "device credential header is not ASCII");
        AdminError::Unauthorized("Invalid device credential".to_owned())
    })?;
    let host_name = headers
        .get("x-systemprompt-host")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AdminError::BadRequest("Missing device host".to_owned()))?;
    let host =
        <systemprompt::models::feedback::EvaluatorClient as serde::Deserialize>::deserialize(
            serde::de::value::StrDeserializer::<serde::de::value::Error>::new(host_name),
        )
        .map_err(|error| {
            tracing::debug!(%error, host_name, "device host is not an evaluator client");
            AdminError::BadRequest("Invalid device host".to_owned())
        })?;
    let identity = repository
        .authenticate_consumer_device(credential)
        .await
        .map_err(|error| {
            tracing::debug!(%error, "device credential rejected");
            AdminError::Unauthorized("Device credential is unavailable or revoked".to_owned())
        })?;
    if &identity.consumer_id != user {
        return Err(AdminError::Forbidden(
            "Hook consumer does not match enrolled device".to_owned(),
        ));
    }
    Ok(Some(
        systemprompt_web_admin_core::repositories::ingestion::webhook::AuthenticatedHookDevice {
            consumer_id: identity.consumer_id,
            device_id: identity.device_id,
            host,
        },
    ))
}
