//! Machine credentials: what a service account presents to a provider.
//!
//! A workflow's write-back acts as a per-definition service account, never as
//! a global key. The account holds at most one credential per provider; the
//! broker exchanges it for a short-lived token only when the caller names a
//! purpose a service account may serve ([`caller`]).
//!
//! The secret half is sealed in an envelope bound to its user and provider,
//! like hosted MCP grants, so a row copied onto another account cannot open.

mod cache;
pub mod caller;
mod exchange;
pub mod input;

use crate::repositories::connectors::machine_credentials::{
    MachineCredentialRow, NewMachineCredential, find_machine_credential, set_machine_credential,
};
use crate::services::service_accounts::ServiceAccounts;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::crypto::secret_crypto;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

pub use cache::forget as forget_cached;
pub use exchange::endpoint_allowed;
pub use input::{AttachCredential, MachineKind};

#[derive(Serialize, Deserialize)]
struct SealedSecret {
    user: String,
    provider: String,
    secret: String,
}

impl std::fmt::Debug for SealedSecret {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SealedSecret { <redacted> }")
    }
}

// Why: attaching to a provider the account already holds rotates it in place
// (`set_machine_credential`), so there is no separate rotate path.
#[derive(Debug, Clone, Copy)]
pub struct Holder<'a> {
    pub user: &'a UserId,
    pub provider: &'a str,
}

pub async fn attach(
    pool: &PgPool,
    accounts: &ServiceAccounts,
    holder: Holder<'_>,
    input: AttachCredential,
    created_by: &str,
) -> AdminResult<()> {
    let Holder { user, provider } = holder;
    if !accounts.is_service_account(user).await? {
        return Err(AdminError::Unprocessable(
            "Only a service account may hold a machine credential".to_owned(),
        ));
    }
    let (kind, config, secret) = input.validated()?;
    let key = secret_crypto::load_master_key()?;
    let nonce = secret_crypto::generate_nonce();
    let envelope = serde_json::to_vec(&SealedSecret {
        user: user.as_str().to_owned(),
        provider: provider.to_owned(),
        secret,
    })
    .map_err(AdminError::internal)?;
    let ciphertext = secret_crypto::encrypt(&key, &nonce, &envelope)?;
    set_machine_credential(
        pool,
        &NewMachineCredential {
            user,
            provider,
            kind: kind.as_str(),
            config,
            secret_ciphertext: ciphertext,
            secret_nonce: nonce.to_vec(),
            created_by,
        },
    )
    .await?;
    cache::forget(user, provider);
    Ok(())
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
    if let Some(value) = cache::get(user, provider, stamp) {
        return Ok(value);
    }
    let secret = open(&row, user, provider)?;
    let kind = MachineKind::parse(&row.kind)?;
    let minted = exchange::mint(kind, &row.config, &secret).await?;
    tracing::info!(target: "connector_broker", user_id = %user, provider,
        caller = caller.unwrap_or(""), kind = kind.as_str(), decision = "issued",
        "service account bearer issued");
    cache::put(user, provider, stamp, &minted);
    Ok(minted.header_value)
}

fn open(row: &MachineCredentialRow, user: &UserId, provider: &str) -> AdminResult<String> {
    let key = secret_crypto::load_master_key()?;
    let nonce = row
        .secret_nonce
        .as_slice()
        .try_into()
        .map_err(AdminError::internal)?;
    let bytes = secret_crypto::decrypt(&key, &nonce, &row.secret_ciphertext)?;
    let sealed: SealedSecret = serde_json::from_slice(&bytes).map_err(AdminError::internal)?;
    if sealed.user != user.as_str() || sealed.provider != provider {
        return Err(AdminError::Forbidden(
            "Machine credential owner mismatch".into(),
        ));
    }
    Ok(sealed.secret)
}
