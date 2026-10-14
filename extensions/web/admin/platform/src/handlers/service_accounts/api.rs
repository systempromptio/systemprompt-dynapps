//! JSON mutations behind the service-account pages: create, delete, and
//! attach / rotate / remove a machine credential. Admin only; every credential
//! change writes a `connector_credential_audit` row naming the admin.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories::connectors::machine_credentials::{
    count_machine_credentials, delete_machine_credential,
};
use crate::repositories::connectors::odoo_user_links::{CredentialAudit, insert_credential_audit};
use crate::repositories::service_account_group::insert_service_account_membership;
use crate::services::connector_oauth::Provider;
use crate::services::machine_credentials::{self, AttachCredential, Holder};
use crate::services::service_accounts::ServiceAccounts;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::types::UserContext;

fn require_admin(user: &UserContext) -> AdminResult<()> {
    if user.is_admin {
        Ok(())
    } else {
        Err(AdminError::Forbidden("Admin access required".to_owned()))
    }
}

async fn require_service_account(accounts: &ServiceAccounts, user_id: &UserId) -> AdminResult<()> {
    if accounts.is_service_account(user_id).await? {
        Ok(())
    } else {
        Err(AdminError::Unprocessable(
            "Only a service account may hold a machine credential".to_owned(),
        ))
    }
}

fn provider_slug(raw: String) -> AdminResult<String> {
    Provider::try_from(raw)
        .map(|p| p.slug().to_owned())
        .map_err(AdminError::BadRequest)
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateServiceAccount {
    pub name: String,
    pub email: String,
}

pub(crate) async fn create(
    Extension(user_ctx): Extension<UserContext>,
    Extension(accounts): Extension<ServiceAccounts>,
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<CreateServiceAccount>,
) -> AdminResult<Response> {
    require_admin(&user_ctx)?;
    let name = body.name.trim();
    let email = body.email.trim();
    if name.is_empty() || name.len() > 128 || !email.contains('@') || email.len() > 254 {
        return Err(AdminError::BadRequest(
            "A service account needs a name and an email address".to_owned(),
        ));
    }
    let user_id = accounts.create(name, email, &user_ctx.user_id).await?;
    insert_service_account_membership(&pool, &user_id, &user_ctx.user_id).await?;
    tracing::info!(target: "connector_broker", actor = %user_ctx.user_id, user_id = %user_id,
        "service account created");
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "user_id": user_id })),
    )
        .into_response())
}

pub(crate) async fn delete(
    Extension(user_ctx): Extension<UserContext>,
    Extension(accounts): Extension<ServiceAccounts>,
    State(pool): State<Arc<PgPool>>,
    Path(user_id): Path<UserId>,
) -> AdminResult<Response> {
    require_admin(&user_ctx)?;
    require_service_account(&accounts, &user_id).await?;
    if count_machine_credentials(&pool, &user_id).await? > 0 {
        return Err(AdminError::Conflict(
            "Remove the account's credentials before deleting it".to_owned(),
        ));
    }
    accounts.delete(&user_id, &user_ctx.user_id).await?;
    tracing::info!(target: "connector_broker", actor = %user_ctx.user_id, user_id = %user_id,
        "service account deleted");
    Ok(StatusCode::NO_CONTENT.into_response())
}

pub(crate) async fn attach_credential(
    Extension(user_ctx): Extension<UserContext>,
    Extension(accounts): Extension<ServiceAccounts>,
    State(pool): State<Arc<PgPool>>,
    Path((user_id, provider)): Path<(UserId, String)>,
    Json(body): Json<AttachCredential>,
) -> AdminResult<Response> {
    require_admin(&user_ctx)?;
    let provider = provider_slug(provider)?;
    let attached = machine_credentials::attach(
        &pool,
        &accounts,
        Holder {
            user: &user_id,
            provider: &provider,
        },
        body,
        user_ctx.user_id.as_str(),
    )
    .await?;
    insert_credential_audit(
        &pool,
        CredentialAudit {
            subject: &user_id,
            provider: &provider,
            action: attached.as_str(),
            actor: &user_ctx.user_id,
        },
    )
    .await?;
    tracing::info!(target: "connector_broker", actor = %user_ctx.user_id, user_id = %user_id,
        provider, action = attached.as_str(), "machine credential attached");
    Ok(StatusCode::NO_CONTENT.into_response())
}

pub(crate) async fn remove_credential(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path((user_id, provider)): Path<(UserId, String)>,
) -> AdminResult<Response> {
    require_admin(&user_ctx)?;
    let provider = provider_slug(provider)?;
    if !delete_machine_credential(&pool, &user_id, &provider).await? {
        return Err(AdminError::NotFound("No such credential".to_owned()));
    }
    insert_credential_audit(
        &pool,
        CredentialAudit {
            subject: &user_id,
            provider: &provider,
            action: "remove",
            actor: &user_ctx.user_id,
        },
    )
    .await?;
    tracing::info!(target: "connector_broker", actor = %user_ctx.user_id, user_id = %user_id,
        provider, "machine credential removed");
    Ok(StatusCode::NO_CONTENT.into_response())
}
