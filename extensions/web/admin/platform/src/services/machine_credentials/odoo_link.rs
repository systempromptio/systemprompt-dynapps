//! A person's own Odoo account: linked from the connectors page, verified
//! against Odoo before it is stored, and opened for the Odoo MCP server.
//!
//! The instance is the profile's `odoo_url` / `odoo_db` secrets; the person
//! supplies only their login and an API key. [`link_odoo_account`] calls
//! Odoo's `common.authenticate` with them and refuses to store a pair Odoo
//! rejects, so a stored link always names the uid it acts as. Every link,
//! re-link and unlink writes a `connector_credential_audit` row naming the
//! actor. The key is sealed under [`ODOO_LINK_PROVIDER`], a different
//! envelope binding than the service-account credential, so neither row can
//! stand in for the other.

use std::time::Duration;

use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

use super::odoo::OdooMachineCredential;
use super::odoo_rpc::{OdooLogin, authenticate};
use super::{endpoint_allowed, seal};
use crate::repositories::connectors::odoo_user_links::{
    CredentialAudit, NewOdooUserLink, delete_odoo_user_link, find_odoo_user_link,
    insert_credential_audit, set_odoo_user_link,
};

pub const ODOO_LINK_PROVIDER: &str = "odoo-user";

#[derive(Deserialize)]
pub struct OdooLinkRequest {
    pub login: String,
    pub api_key: String,
}

impl std::fmt::Debug for OdooLinkRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OdooLinkRequest")
            .field("login", &self.login)
            .finish_non_exhaustive()
    }
}

fn secret(name: &str) -> Option<String> {
    systemprompt::config::SecretsBootstrap::get()
        .ok()
        .and_then(|s| s.get(name).cloned())
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}

fn instance() -> AdminResult<(String, String)> {
    let (Some(url), Some(db)) = (secret("odoo_url"), secret("odoo_db")) else {
        return Err(AdminError::Unavailable(
            "Odoo is not configured on this instance (odoo_url / odoo_db)".to_owned(),
        ));
    };
    let url = url.trim_end_matches('/').to_owned();
    if !endpoint_allowed(&url) {
        return Err(AdminError::Unavailable(
            "odoo_url is not an HTTPS origin".to_owned(),
        ));
    }
    Ok((url, db))
}

fn field<'a>(value: &'a str, what: &str) -> AdminResult<&'a str> {
    let value = value.trim();
    if value.is_empty() || value.len() > 512 || value.chars().any(char::is_control) {
        return Err(AdminError::BadRequest(format!("{what} is required")));
    }
    Ok(value)
}

pub async fn link_odoo_account(
    pool: &PgPool,
    user: &UserId,
    request: OdooLinkRequest,
    actor: &UserId,
) -> AdminResult<i64> {
    let (url, db) = instance()?;
    let login = field(&request.login, "Odoo login")?;
    let api_key = field(&request.api_key, "Odoo API key")?;
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(AdminError::internal)?;
    let check = OdooLogin {
        url: &url,
        db: &db,
        login,
        api_key,
    };
    let Some(uid) = authenticate(&http, check).await? else {
        tracing::warn!(target: "connector_broker", user_id = %user, actor = %actor,
            decision = "refused", "Odoo link refused: login or API key rejected by Odoo");
        return Err(AdminError::Unprocessable(
            "Odoo rejected that login and API key".to_owned(),
        ));
    };
    let sealed = seal::seal(user, ODOO_LINK_PROVIDER, api_key.to_owned())?;
    let inserted = set_odoo_user_link(
        pool,
        &NewOdooUserLink {
            user,
            odoo_url: &url,
            odoo_db: &db,
            odoo_login: login,
            odoo_uid: uid,
            api_key_ciphertext: sealed.ciphertext,
            api_key_nonce: sealed.nonce,
        },
    )
    .await?;
    let action = if inserted { "link" } else { "relink" };
    audit(pool, user, action, actor).await?;
    tracing::info!(target: "connector_broker", user_id = %user, actor = %actor, odoo_uid = uid,
        action, "Odoo account linked");
    Ok(uid)
}

pub async fn unlink_odoo_account(
    pool: &PgPool,
    user: &UserId,
    actor: &UserId,
) -> AdminResult<bool> {
    let removed = delete_odoo_user_link(pool, user).await?;
    if removed {
        audit(pool, user, "unlink", actor).await?;
        tracing::info!(target: "connector_broker", user_id = %user, actor = %actor,
            "Odoo account unlinked");
    }
    Ok(removed)
}

async fn audit(pool: &PgPool, user: &UserId, action: &str, actor: &UserId) -> AdminResult<()> {
    insert_credential_audit(
        pool,
        CredentialAudit {
            subject: user,
            provider: ODOO_LINK_PROVIDER,
            action,
            actor,
        },
    )
    .await?;
    Ok(())
}

pub async fn odoo_user_credential_for(
    pool: &PgPool,
    user_id: &UserId,
) -> AdminResult<Option<OdooMachineCredential>> {
    let Some(row) = find_odoo_user_link(pool, user_id).await? else {
        return Ok(None);
    };
    let api_key = seal::unseal(
        &row.api_key_ciphertext,
        &row.api_key_nonce,
        user_id,
        ODOO_LINK_PROVIDER,
    )?;
    Ok(Some(OdooMachineCredential {
        url: row.odoo_url,
        db: row.odoo_db,
        login: row.odoo_login,
        api_key,
    }))
}
