//! HTTP handlers for secret storage, rotation, and short-lived resolution
//! tokens.

use axum::Extension;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;

use systemprompt::identifiers::{PluginId, UserId};

use crate::services::auth::validate_plugin_jwt;
use crate::services::secret_service;
use systemprompt_web_admin_core::auth::session::extract_user_from_cookie;
use systemprompt_web_admin_core::error::AdminResult;
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;

use systemprompt_web_admin_core::types::responses::{
    AuditLogEntry, AuditLogListResponse, ResolutionTokenResponse, ResultOkResponse,
    SecretsListResponse,
};

const RESOLUTION_TOKEN_EXPIRY_SECS: u32 = 300;

#[derive(serde::Deserialize, Debug)]
pub(crate) struct ResolveQuery {
    token: String,
}

pub(crate) async fn create_resolution_token_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<PluginId>,
    Extension(boot): Extension<ExtensionBoot>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    let subject = validate_plugin_jwt(boot.token_authority().as_ref(), &headers)?;
    let user_id = UserId::new(&subject);
    let token = secret_service::create_resolution_token(&pool, &user_id, &plugin_id).await?;
    Ok(Json(ResolutionTokenResponse {
        token,
        expires_in: RESOLUTION_TOKEN_EXPIRY_SECS,
    })
    .into_response())
}

pub(crate) async fn resolve_secrets_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<PluginId>,
    Extension(boot): Extension<ExtensionBoot>,
    Query(params): Query<ResolveQuery>,
) -> AdminResult<Response> {
    let secrets =
        secret_service::resolve_secrets(&pool, boot.secrets().as_ref(), &plugin_id, &params.token)
            .await?;
    Ok(Json(SecretsListResponse { secrets }).into_response())
}

pub(crate) async fn audit_log_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<PluginId>,
    Extension(boot): Extension<ExtensionBoot>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    let session = extract_user_from_cookie(boot.token_authority().as_ref(), &headers)?;
    let entries = secret_service::list_audit_log(&pool, &session.user_id, &plugin_id).await?;
    let items: Vec<AuditLogEntry> = entries
        .into_iter()
        .map(|row| AuditLogEntry {
            id: row.id,
            var_name: row.var_name,
            action: row.action,
            actor_id: row.actor_id.as_str().to_owned(),
            ip_address: row.ip_address,
            created_at: row.created_at,
        })
        .collect();
    Ok(Json(AuditLogListResponse { entries: items }).into_response())
}

pub(crate) async fn rotate_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<PluginId>,
    Extension(boot): Extension<ExtensionBoot>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    let session = extract_user_from_cookie(boot.token_authority().as_ref(), &headers)?;
    secret_service::rotate_user_keys(&pool, boot.secrets().as_ref(), &session.user_id, &plugin_id)
        .await?;
    Ok(Json(ResultOkResponse { result: "ok" }).into_response())
}
