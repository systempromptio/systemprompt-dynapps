//! HTTP handlers for registered device management.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories;
use crate::services::device_service;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::state::ManagedState;
use systemprompt_web_admin_core::types::UserContext;

#[derive(Debug, Deserialize)]
pub(crate) struct IssueApiKeyRequest {
    pub name: String,
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
}

pub(crate) async fn issue_pat(
    Extension(user_ctx): Extension<UserContext>,
    Extension(managed): Extension<Arc<ManagedState>>,
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<IssueApiKeyRequest>,
) -> AdminResult<Response> {
    let issued = device_service::issue_pat(
        &pool,
        managed.security,
        &user_ctx.user_id,
        &body.name,
        body.expires_at,
    )
    .await?;
    Ok(Json(issued).into_response())
}

pub(crate) async fn revoke_pat(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> AdminResult<Response> {
    device_service::revoke_pat(&pool, &user_ctx.user_id, &id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[derive(Debug, Deserialize)]
pub(crate) struct EnrollDeviceRequest {
    pub user_id: UserId,
    pub name: String,
    pub platform: String,
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
}

pub(crate) async fn enroll_device(
    Extension(user_ctx): Extension<UserContext>,
    Extension(managed): Extension<Arc<ManagedState>>,
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<EnrollDeviceRequest>,
) -> AdminResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }
    let target = body.user_id;
    let hostname = body.hostname.unwrap_or_default();
    let enrolled = device_service::enroll_device(
        &pool,
        managed.security,
        &target,
        device_service::EnrollDeviceInput {
            name: &body.name,
            platform: &body.platform,
            hostname: &hostname,
            expires_at: body.expires_at,
        },
    )
    .await?;
    Ok((StatusCode::CREATED, Json(enrolled)).into_response())
}

pub(crate) async fn revoke_cert(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> AdminResult<Response> {
    device_service::revoke_device_cert(&pool, &user_ctx.user_id, &id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

pub(crate) async fn admin_revoke_credential(
    State(pool): State<Arc<PgPool>>,
    Path((kind, id)): Path<(String, String)>,
) -> AdminResult<Response> {
    let revoked = match kind.as_str() {
        "pats" => repositories::devices::pats::revoke_any_api_key(&pool, &id).await,
        "certs" => repositories::devices::certs::revoke_any_device_cert(&pool, &id).await,
        _ => {
            return Err(AdminError::NotFound(format!(
                "unknown credential kind: {kind}"
            )));
        },
    }?;

    if revoked {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Err(AdminError::NotFound(
            "no active credential with that id".to_owned(),
        ))
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct SetCertExpiryRequest {
    #[serde(default)]
    pub valid_until: Option<DateTime<Utc>>,
}

pub(crate) async fn admin_set_cert_expiry(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
    Json(body): Json<SetCertExpiryRequest>,
) -> AdminResult<Response> {
    if body.valid_until.is_some_and(|until| until <= Utc::now()) {
        return Err(AdminError::BadRequest(
            "valid_until must be in the future".to_owned(),
        ));
    }
    let set = repositories::devices::certs::set_device_cert_validity(&pool, &id, body.valid_until)
        .await?;
    if set {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Err(AdminError::NotFound(
            "no active certificate with that id".to_owned(),
        ))
    }
}
