//! Machine credentials: what a service account presents to a provider.
//!
//! A workflow's write-back acts as a per-definition service account, never as
//! a global key. The account holds at most one credential per provider; the
//! broker exchanges it for a short-lived token only when the caller names a
//! purpose a service account may serve ([`caller`]). Minting and its cache
//! are core's (the private `grant` module); this module stores, seals and
//! gates.
//!
//! The secret half is sealed in an envelope bound to its user and provider,
//! like hosted MCP grants, so a row copied onto another account cannot open.

pub mod caller;
mod grant;
pub mod input;
mod odoo;
mod odoo_link;
mod odoo_rpc;
mod seal;

use crate::repositories::connectors::machine_credentials::{
    MachineCredentialRow, NewMachineCredential, find_machine_credential, set_machine_credential,
};
use crate::services::service_accounts::ServiceAccounts;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

pub use grant::endpoint_allowed;
pub use input::{AttachCredential, MachineKind};
pub use odoo::{
    ODOO_PROVIDER, OdooCredentialLookup, OdooMachineCredential, odoo_credential_for,
    resolve_odoo_credential,
};
pub use odoo_link::{
    ODOO_LINK_PROVIDER, OdooLinkRequest, link_odoo_account, odoo_user_credential_for,
    unlink_odoo_account,
};

// Why: attaching to a provider the account already holds rotates it in place
// (`set_machine_credential`), so there is no separate rotate path.
#[derive(Debug, Clone, Copy)]
pub struct Holder<'a> {
    pub user: &'a UserId,
    pub provider: &'a str,
}

/// What an attach did: a first credential for the provider, or a rotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attached {
    New,
    Rotated,
}

impl Attached {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::New => "attach",
            Self::Rotated => "rotate",
        }
    }
}

pub async fn attach(
    pool: &PgPool,
    accounts: &ServiceAccounts,
    holder: Holder<'_>,
    input: AttachCredential,
    created_by: &str,
) -> AdminResult<Attached> {
    let Holder { user, provider } = holder;
    if !accounts.is_service_account(user).await? {
        return Err(AdminError::Unprocessable(
            "Only a service account may hold a machine credential".to_owned(),
        ));
    }
    let (kind, config, secret) = input.validated()?;
    let existed = find_machine_credential(pool, user, provider)
        .await?
        .is_some();
    let sealed = seal::seal(user, provider, secret)?;
    set_machine_credential(
        pool,
        &NewMachineCredential {
            user,
            provider,
            kind: kind.as_str(),
            config,
            secret_ciphertext: sealed.ciphertext,
            secret_nonce: sealed.nonce,
            created_by,
        },
    )
    .await?;
    Ok(if existed {
        Attached::Rotated
    } else {
        Attached::New
    })
}

pub async fn broker_token(
    pool: &PgPool,
    user: &UserId,
    provider: &str,
    caller: Option<&str>,
) -> AdminResult<String> {
    if !caller::allowed(caller) {
        tracing::warn!(target: "connector_broker", user_id = %user, provider,
            caller = caller.unwrap_or(""), decision = "refused",
            "service account bearer refused: caller purpose not permitted");
        return Err(AdminError::Forbidden(
            "A service account credential is only issued to a workflow write-back or farm caller"
                .into(),
        ));
    }
    let row = find_machine_credential(pool, user, provider)
        .await?
        .ok_or_else(|| AdminError::NotFound("Service account holds no credential".into()))?;
    let stamp = row.rotated_at.unwrap_or(row.created_at).timestamp_micros();
    let kind = MachineKind::parse(&row.kind)?;
    let grant = grant::grant(kind, &row.config, open(&row, user, provider)?)?;
    // Why: core's token cache is keyed by string; the rotation stamp in the
    // key means a rotated credential never serves the previous token.
    let header_value = grant
        .authorization(&format!("service-account:{user}:{provider}:{stamp}"))
        .await
        .map_err(|error| {
            tracing::warn!(target: "connector_broker", user_id = %user, provider,
                kind = kind.as_str(), %error, decision = "failed",
                "service account bearer could not be minted");
            AdminError::Upstream(format!("{} token exchange failed", kind.label()))
        })?;
    tracing::info!(target: "connector_broker", user_id = %user, provider,
        caller = caller.unwrap_or(""), kind = kind.as_str(), decision = "issued",
        "service account bearer issued");
    Ok(header_value)
}

fn open(row: &MachineCredentialRow, user: &UserId, provider: &str) -> AdminResult<String> {
    seal::unseal(&row.secret_ciphertext, &row.secret_nonce, user, provider)
}
