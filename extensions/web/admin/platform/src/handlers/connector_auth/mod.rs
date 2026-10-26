//! Hosted MCP consent and account actions; credentials stay in the backend.

pub(crate) mod account;
mod completion;
pub(crate) mod consent;
mod deps;
pub(crate) mod identity;
pub(crate) mod token;

use crate::repositories::connectors::{
    connector_accounts as accounts, connector_credentials as credentials,
};
use crate::services::connector_oauth as oauth;
use axum::http::HeaderMap;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt::traits::SecretStore;
use systemprompt_web_admin_core::auth::session::extract_mcp_accessor_user;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;

fn require_broker(secrets: &dyn SecretStore, headers: &HeaderMap) -> AdminResult<()> {
    use sha2::{Digest, Sha256};
    let expected = oauth::config::secret(secrets, "mcp_credential_broker_secret")?;
    let presented = headers
        .get("x-systemprompt-credential-broker")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AdminError::Forbidden("Backend credential access required".into()))?;
    if Sha256::digest(expected.as_bytes()) != Sha256::digest(presented.as_bytes()) {
        return Err(AdminError::Forbidden(
            "Backend credential access rejected".into(),
        ));
    }
    Ok(())
}

async fn live_user(
    pool: &PgPool,
    boot: &ExtensionBoot,
    headers: &HeaderMap,
    mutation: bool,
) -> AdminResult<systemprompt_web_admin_core::types::CookieSession> {
    let rides_ambient_cookies = !headers.contains_key("authorization");
    if mutation && rides_ambient_cookies {
        let origin = headers
            .get("origin")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| AdminError::Forbidden("Same-origin request required".into()))?;
        let expected = url::Url::parse(&boot.config().api_external_url)
            .map_err(AdminError::internal)?
            .origin()
            .ascii_serialization();
        if origin != expected {
            return Err(AdminError::Forbidden(
                "Cross-origin account mutation rejected".into(),
            ));
        }
    }
    let user = extract_mcp_accessor_user(boot.token_authority().as_ref(), headers)?;
    let session = user
        .session_id
        .as_ref()
        .ok_or_else(|| AdminError::Unauthorized("A live login session is required".into()))?;
    if !accounts::is_live_session(pool, &user.user_id, session.as_str()).await? {
        return Err(AdminError::Unauthorized(
            "Login session is no longer active".into(),
        ));
    }
    Ok(user)
}

async fn store_verified(
    pool: &PgPool,
    secrets: &dyn SecretStore,
    user: &UserId,
    grant: &oauth::Grant,
) -> AdminResult<()> {
    let mut tx = pool.begin().await?;
    let mut row = accounts::get_locked_account(&mut tx, user, grant.provider.slug()).await?;
    if row.generation != grant.generation {
        return Err(AdminError::Conflict(
            "Connection changed during authorization; start again".into(),
        ));
    }
    credentials::store(
        &mut tx,
        user,
        grant.provider.slug(),
        &oauth::seal(secrets, grant)?,
    )
    .await?;
    account::apply_verified(&mut row, grant);
    row.generation += 1;
    accounts::update_account(&mut tx, user, &row).await?;
    accounts::delete_pending(&mut tx, user, grant.provider.slug()).await?;
    tx.commit().await?;
    Ok(())
}

async fn store_pending_verification(
    pool: &PgPool,
    secrets: &dyn SecretStore,
    user: &UserId,
    grant: &oauth::Grant,
) -> AdminResult<()> {
    let mut tx = pool.begin().await?;
    let mut row = accounts::get_locked_account(&mut tx, user, grant.provider.slug()).await?;
    if row.generation != grant.generation {
        return Err(AdminError::Conflict(
            "Connection changed during authorization; start again".into(),
        ));
    }
    credentials::store(
        &mut tx,
        user,
        grant.provider.slug(),
        &oauth::seal(secrets, grant)?,
    )
    .await?;
    row.status = "verification_required".into();
    row.auth_method = Some(grant.auth_method.clone());
    row.account_id = None;
    row.account_name = None;
    row.resource_id = None;
    row.resource_name = None;
    row.verified_at = None;
    row.error_code = Some("verification_failed".into());
    row.generation += 1;
    accounts::update_account(&mut tx, user, &row).await?;
    accounts::delete_pending(&mut tx, user, grant.provider.slug()).await?;
    tx.commit().await?;
    Ok(())
}
