//! The Odoo API key a service account holds, opened for the in-process Odoo
//! MCP server.
//!
//! Odoo's JSON-RPC takes the key as the password on every call, so there is
//! no token to broker: the server asks for the opened key directly. It must
//! still gate on the caller's purpose ([`super::caller::allowed`]) before it
//! spends the key. [`odoo_credential_for`] answers `Some` only when the user
//! is an active service account holding an `odoo_api_key` credential under
//! the `odoo` provider; the URL is re-checked as an origin on every open.
//!
//! [`resolve_odoo_credential`] is the one entry point for both kinds of
//! caller: a service account resolves to its machine credential and never to
//! a person link, and a person resolves to the account they linked
//! ([`super::odoo_link`]). A caller holding neither is told which remedy
//! applies to their kind.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::error::AdminResult;

use super::{MachineKind, grant, open};
use crate::repositories::connectors::machine_credentials::find_machine_credential;
use crate::services::service_accounts::ServiceAccounts;

pub const ODOO_PROVIDER: &str = "odoo";

#[derive(Debug, Clone)]
pub enum OdooCredentialLookup {
    ServiceAccount(OdooMachineCredential),
    Person(OdooMachineCredential),
    ServiceAccountMissing,
    PersonUnlinked,
}

pub async fn resolve_odoo_credential(
    pool: &PgPool,
    accounts: &ServiceAccounts,
    user_id: &UserId,
) -> AdminResult<OdooCredentialLookup> {
    if accounts.is_service_account(user_id).await? {
        return Ok(machine_credential(pool, user_id).await?.map_or(
            OdooCredentialLookup::ServiceAccountMissing,
            OdooCredentialLookup::ServiceAccount,
        ));
    }
    Ok(super::odoo_link::odoo_user_credential_for(pool, user_id)
        .await?
        .map_or(
            OdooCredentialLookup::PersonUnlinked,
            OdooCredentialLookup::Person,
        ))
}

#[derive(Clone)]
pub struct OdooMachineCredential {
    pub url: String,
    pub db: String,
    pub login: String,
    pub api_key: String,
}

impl std::fmt::Debug for OdooMachineCredential {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OdooMachineCredential")
            .field("url", &self.url)
            .field("db", &self.db)
            .field("login", &self.login)
            .finish_non_exhaustive()
    }
}

pub async fn odoo_credential_for(
    pool: &PgPool,
    accounts: &ServiceAccounts,
    user_id: &UserId,
) -> AdminResult<Option<OdooMachineCredential>> {
    if !accounts.is_service_account(user_id).await? {
        return Ok(None);
    }
    machine_credential(pool, user_id).await
}

async fn machine_credential(
    pool: &PgPool,
    user_id: &UserId,
) -> AdminResult<Option<OdooMachineCredential>> {
    let Some(row) = find_machine_credential(pool, user_id, ODOO_PROVIDER).await? else {
        return Ok(None);
    };
    if MachineKind::parse(&row.kind)? != MachineKind::OdooApiKey {
        return Ok(None);
    }
    let fields = grant::odoo_fields(&row.config)?;
    Ok(Some(OdooMachineCredential {
        url: fields.0,
        db: fields.1,
        login: fields.2,
        api_key: open(&row, user_id, ODOO_PROVIDER)?,
    }))
}
