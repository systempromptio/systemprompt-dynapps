//! Per-run gateway credentials minted for the run owner and revoked after.
//!
//! An ephemeral PAT, the gateway session `/v1/messages` requires as
//! `x-session-id`, and a best-effort hook token for the governance plugin.
//! The PAT secret lives only in memory and in the child's environment.

mod hook_token;
mod session;

use std::fmt;
use std::time::Duration;

use sqlx::PgPool;
use systemprompt::identifiers::{PluginId, UserId};
use systemprompt_web_admin_core::repositories::bridge::api_keys::{issue_api_key, revoke_api_key};

use crate::error::{FarmError, FarmResult};
use crate::state::GatewayEndpoints;
use crate::types::{FarmId, FarmRunId};

pub use hook_token::mint_hook_token;
pub use session::mint_gateway_session;

pub struct RunCredentials {
    pub api_key_id: String,
    pub pat_secret: String,
    pub gateway_session_id: String,
    pub hook_token: Option<String>,
    pub hook_token_error: Option<String>,
}

impl fmt::Debug for RunCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RunCredentials")
            .field("api_key_id", &self.api_key_id)
            .field("gateway_session_id", &self.gateway_session_id)
            .field(
                "hook_token",
                &self.hook_token.as_ref().map(|_| "<redacted>"),
            )
            .field("hook_token_error", &self.hook_token_error)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CredentialRequest<'a> {
    pub user_id: &'a UserId,
    pub farm_id: &'a FarmId,
    pub run_id: &'a FarmRunId,
    pub plugin_id: &'a PluginId,
    pub ttl: Duration,
}

pub async fn issue_run_credentials(
    pool: &PgPool,
    http: &reqwest::Client,
    endpoints: &GatewayEndpoints,
    req: CredentialRequest<'_>,
) -> FarmResult<RunCredentials> {
    let name = format!("agent-farm:{}:{}", req.farm_id, req.run_id);
    let ttl = chrono::Duration::from_std(req.ttl).unwrap_or_else(|_| chrono::Duration::hours(1));
    let issued = issue_api_key(pool, req.user_id, &name, Some(chrono::Utc::now() + ttl))
        .await
        .map_err(|source| FarmError::Credential {
            step: "issue PAT",
            source,
        })?;
    let base = endpoints.internal_base.as_str();

    let gateway_session_id = match mint_gateway_session(http, base, &issued.secret).await {
        Ok(id) => id,
        Err(err) => {
            if let Err(revoke_err) = revoke_api_key(pool, req.user_id, &issued.id).await {
                tracing::warn!(error = %revoke_err, key = %issued.id, "farm PAT revoke failed");
            }
            return Err(err);
        },
    };

    let (hook_token, hook_token_error) = match mint_hook_token(
        http,
        base,
        &issued.secret,
        &gateway_session_id,
        req.plugin_id,
    )
    .await
    {
        Ok(token) => (Some(token), None),
        Err(err) => (None, Some(err.to_string())),
    };

    Ok(RunCredentials {
        api_key_id: issued.id,
        pat_secret: issued.secret,
        gateway_session_id,
        hook_token,
        hook_token_error,
    })
}

pub async fn revoke_run_credentials(
    pool: &PgPool,
    user_id: &UserId,
    api_key_id: &str,
) -> FarmResult<bool> {
    revoke_api_key(pool, user_id, api_key_id)
        .await
        .map_err(|source| FarmError::Credential {
            step: "revoke PAT",
            source,
        })
}
