//! Plugin JWT validation for the admin handlers.

use axum::http::HeaderMap;
use systemprompt::models::auth::{JwtAudience, TokenAuthority};
use systemprompt::oauth::validate_jwt_token;

use systemprompt_web_admin_core::error::{AdminError, AdminResult};

pub(crate) fn validate_plugin_jwt(
    authority: &dyn TokenAuthority,
    headers: &HeaderMap,
) -> AdminResult<String> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AdminError::Unauthorized("Missing Authorization header".to_owned()))?;

    let claims = validate_jwt_token(
        authority,
        token,
        &[JwtAudience::Resource("plugin".to_owned())],
    )
    .map_err(|e| {
        tracing::warn!(error = %e, "Plugin JWT validation failed");
        AdminError::Unauthorized("Invalid or expired token".to_owned())
    })?;

    Ok(claims.registered.sub)
}
