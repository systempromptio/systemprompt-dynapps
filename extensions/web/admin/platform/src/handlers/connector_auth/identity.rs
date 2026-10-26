//! Broker-only accessor that signs the caller's identity for one external
//! server.

use super::{live_user, require_broker};
use crate::services::identity_token;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::http::header::CACHE_CONTROL;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::governance::authz::EntityKind;
use systemprompt_web_admin_core::authz::catalog::CatalogAccess;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;
use systemprompt_web_admin_core::repositories::principal::identity::find_identity_envelope;

pub(crate) async fn token(
    Extension(services): Extension<
        systemprompt_web_admin_core::extension_services::ExtensionServices,
    >,
    State(pool): State<Arc<PgPool>>,
    Extension(boot): Extension<ExtensionBoot>,
    Path(server): Path<String>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    require_broker(boot.secrets().as_ref(), &headers)?;
    let user = live_user(&pool, &boot, &headers, false).await?;
    let audience = identity_token::audience_for(&services.config(), &server)
        .ok_or_else(|| AdminError::NotFound("No identity-backed MCP server by that id".into()))?;
    let permitted = CatalogAccess::load(&pool, &services, &user.user_id)
        .await?
        .allowed(EntityKind::McpServer, std::slice::from_ref(&server))
        .await?;
    if !permitted.contains(&server) {
        return Err(AdminError::Forbidden(
            "This account is not entitled to the MCP server".into(),
        ));
    }
    let identity = find_identity_envelope(&pool, &user.user_id)
        .await?
        .ok_or_else(|| AdminError::Unauthorized("Account unavailable".into()))?;
    let name = identity
        .display_name
        .filter(|n| !n.trim().is_empty())
        .unwrap_or(identity.username);
    let claims = identity_token::claims(
        &boot.config().jwt_issuer,
        &audience,
        &user.user_id,
        &identity.email,
        &name,
    );
    let access_token = identity_token::sign(boot.token_authority().as_ref(), &claims)?;
    Ok((
        [(CACHE_CONTROL, "no-store")],
        Json(serde_json::json!({"access_token": access_token})),
    )
        .into_response())
}
