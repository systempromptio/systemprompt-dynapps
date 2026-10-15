//! Account-row state transitions for a connector: how a failure, a rejected
//! grant or a successful probe is written back to `mcp_connector_accounts`.
//! Only a rejected grant is deleted; after an outage the account is held for
//! `OUTAGE_HOLD_SECONDS` so accessor calls do not refresh once per MCP request.

use super::failure::{FailureKind, FlowFailure};
use super::{Grant, Provider, seal};
use crate::repositories::connectors::connector_accounts::{ProviderConnection, update_account};
use crate::repositories::connectors::connector_credentials::{self as repo, EncryptedGrant};
use chrono::Utc;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

pub(super) async fn record_grant_failure(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user: &UserId,
    account: &mut ProviderConnection,
    grant: &Grant,
    failure: &FlowFailure,
) -> AdminResult<()> {
    let provider = &grant.provider;
    if failure.kind == FailureKind::GrantRejected {
        account.status = "reconnect_required".into();
        account.error_code = Some("grant_rejected".into());
        account.generation += 1;
        repo::delete(tx, user, provider.slug()).await?;
    } else {
        account.status = "temporarily_unavailable".into();
        account.error_code = Some(failure.kind.code().into());
        if matches!(
            failure.kind,
            FailureKind::RefreshUncertain | FailureKind::InvalidResponse | FailureKind::Permission
        ) {
            account.status = "verification_required".into();
            account.verified_at = None;
        }
        let possibly_rotated_grant = seal(grant)?;
        repo::store(tx, user, provider.slug(), &possibly_rotated_grant).await?;
    }
    update_account(tx, user, account).await?;
    tracing::warn!(
        target: "connector_oauth",
        provider = provider.slug(),
        status = %account.status,
        error_code = account.error_code.as_deref().unwrap_or(""),
        generation = account.generation,
        user_id = %user,
        classification = failure.kind.code(),
        error = %failure.error,
        "connector grant failure recorded"
    );
    Ok(())
}

pub(super) async fn open_or_flag(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user: &UserId,
    provider: &Provider,
    row: &EncryptedGrant,
    account: &mut ProviderConnection,
) -> AdminResult<Grant> {
    match super::open(row, user, provider) {
        Ok(grant) => Ok(grant),
        Err(error) => {
            if matches!(error, AdminError::Unauthorized(_)) {
                account.status = "reconnect_required".into();
                account.error_code = Some("configuration_changed".into());
                update_account(tx, user, account).await?;
            }
            Err(error)
        },
    }
}

pub(super) const OUTAGE_HOLD_SECONDS: i64 = 30;

pub(super) fn in_outage_hold(
    account: &ProviderConnection,
    last_attempt: Option<chrono::DateTime<Utc>>,
) -> bool {
    account.status == "temporarily_unavailable"
        && last_attempt.is_some_and(|at| {
            Utc::now().signed_duration_since(at).num_seconds() < OUTAGE_HOLD_SECONDS
        })
}

pub(super) fn token_state(grant: &Grant) -> String {
    if grant.expires_at == i64::MAX {
        return "Personal token on file".to_owned();
    }
    let minutes = (grant.expires_at - Utc::now().timestamp()) / 60;
    format!("Access token valid for {minutes} min")
}

pub(super) fn require_connection(status: &str) -> AdminResult<()> {
    if matches!(status, "not_connected" | "reconnect_required") {
        return Err(AdminError::NotFound(
            "Connect your provider account in Systemprompt".into(),
        ));
    }
    Ok(())
}

pub(super) fn apply_verification(account: &mut ProviderConnection, grant: &Grant) {
    account.auth_method = Some(grant.auth_method.clone());
    account.account_id = Some(grant.account_id.clone());
    account.account_name = Some(grant.account_name.clone());
    account.resource_id = Some(grant.resource_id.clone());
    account.resource_name = Some(grant.resource_name.clone());
    account.status = "connected".into();
    account.error_code = None;
    account.verified_at = Some(Utc::now());
}
