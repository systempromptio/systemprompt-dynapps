//! Hosted MCP authorization, with encrypted user-bound grants and locked
//! refresh.
//!
//! `verified_token` resolves a server-only Authorization header. A probe that
//! fails at a stage is a result, not an error — the Connectors page shows
//! which stage — so `probe_connection` only propagates a failure raised before
//! the first stage (no grant, not configured).

mod account_state;
pub mod callback_notice;
pub mod config;
pub mod discovery;
pub mod failure;
pub mod generic;
mod generic_discovery;
pub mod payload;
mod resolve;
mod response;
mod salesforce;
pub mod site;
pub mod token_plan;
mod tokens;
mod transport;
mod userinfo;
pub mod verify;

use crate::repositories::connectors::connector_credentials::EncryptedGrant;
pub use config::Provider;
pub use generic::Consent;
use resolve::resolve_token;
#[doc(hidden)]
pub use resolve::resolve_token_with_client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::connections::report::VerificationReport;
use systemprompt_web_admin_core::crypto::secret_crypto;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
pub(crate) use tokens::exchange_classified;
pub use transport::{authorize, exchange, exchange_with_client, refresh_with_client};

/// Debug output never reveals credentials; serialization is only for the
/// encrypted credential store, never an HTTP response.
#[derive(Deserialize, Serialize)]
pub struct Grant {
    pub user: String,
    #[serde(default)]
    pub configuration_binding: String,
    #[serde(default)]
    pub authorization_issuer: String,
    #[serde(default)]
    pub token_auth_method: String,
    pub provider: Provider,
    pub client: String,
    pub client_secret: String,
    pub verifier: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: i64,
    #[serde(default)]
    pub token_endpoint: String,
    #[serde(default)]
    pub generation: i64,
    #[serde(default)]
    pub session: Option<String>,
    #[serde(default)]
    pub auth_method: String,
    #[serde(default)]
    pub account_id: String,
    #[serde(default)]
    pub account_name: String,
    #[serde(default)]
    pub resource_id: String,
    #[serde(default)]
    pub resource_name: String,
    #[serde(default = "bearer_scheme")]
    pub authorization_scheme: String,
}

impl std::fmt::Debug for Grant {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Grant { <redacted> }")
    }
}

pub fn seal(grant: &Grant) -> AdminResult<EncryptedGrant> {
    let key = secret_crypto::load_master_key()?;
    let nonce = secret_crypto::generate_nonce();
    let bytes = serde_json::to_vec(grant).map_err(AdminError::internal)?;
    Ok(EncryptedGrant {
        ciphertext: secret_crypto::encrypt(&key, &nonce, &bytes)?,
        nonce: nonce.to_vec(),
    })
}

pub fn open(row: &EncryptedGrant, user: &UserId, provider: &Provider) -> AdminResult<Grant> {
    let key = secret_crypto::load_master_key()?;
    let nonce = row
        .nonce
        .as_slice()
        .try_into()
        .map_err(AdminError::internal)?;
    let bytes = secret_crypto::decrypt(&key, &nonce, &row.ciphertext)?;
    let grant: Grant = serde_json::from_slice(&bytes).map_err(AdminError::internal)?;
    if !grant_bound_to(&grant, user, provider) {
        return Err(AdminError::Forbidden(
            "Connector credential owner mismatch".into(),
        ));
    }
    generic::validate_grant(&grant)?;
    Ok(grant)
}

fn grant_bound_to(grant: &Grant, user: &UserId, provider: &Provider) -> bool {
    grant.user == user.as_str() && grant.provider.slug() == provider.slug()
}

pub async fn verified_token(
    pool: &PgPool,
    user: &UserId,
    provider: Provider,
    probe: bool,
) -> AdminResult<String> {
    let mut report = VerificationReport::for_provider(provider.slug());
    resolve_token(pool, user, provider, probe, &mut report).await
}

pub async fn probe_connection(
    pool: &PgPool,
    user: &UserId,
    provider: Provider,
) -> AdminResult<VerificationReport> {
    let mut report = VerificationReport::for_provider(provider.slug());
    match resolve_token(pool, user, provider, true, &mut report).await {
        Err(error) if report.steps.is_empty() => Err(error),
        Ok(_) | Err(_) => Ok(report.finish()),
    }
}

#[doc(hidden)]
pub async fn refresh_at(grant: &mut Grant, endpoint: &str) -> AdminResult<()> {
    let previous = std::mem::replace(&mut grant.token_endpoint, endpoint.to_owned());
    let result = transport::refresh(grant)
        .await
        .map_err(|failure| failure.error);
    grant.token_endpoint = previous;
    result
}

fn bearer_scheme() -> String {
    "Bearer".into()
}
