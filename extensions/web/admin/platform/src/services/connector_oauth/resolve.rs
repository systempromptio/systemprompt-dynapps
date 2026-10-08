//! Token resolution under the account lock. One transaction carries the
//! account row, the sealed grant, any re-mint or refresh and the state the
//! outcome writes back, so refresh, generation checks and error-state commits
//! land together.
//!
//! A grant that fails to open commits its reconnect flag before the error
//! propagates; a missing grant or a failed first mint rolls back.

use super::failure::{FailureKind, FlowFailure, FlowResult};
use chrono::Utc;
use sqlx::{PgPool, Postgres, Transaction};
use systemprompt::identifiers::UserId;
use tracing::Instrument;

use super::account_state::{
    apply_verification, in_outage_hold, open_or_flag, record_grant_failure, require_connection,
    token_state,
};
use super::token_plan::{AccessMode, TokenPlan, needs_remint};
use super::{Grant, Provider, salesforce, seal, transport, verify};
use crate::repositories::connectors::connector_accounts::{self as accounts, ProviderConnection};
use crate::repositories::connectors::connector_credentials::{self as repo, EncryptedGrant};
use systemprompt_web_admin_core::connections::report::VerificationReport;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

type Tx = Transaction<'static, Postgres>;

/// A user's token broker with an explicitly supplied transport.
#[derive(Debug)]
pub struct TokenResolver<'a> {
    pool: &'a PgPool,
    user: &'a UserId,
    http: &'a reqwest::Client,
}

struct Resolution<'a> {
    pool: &'a PgPool,
    user: &'a UserId,
    provider: &'a Provider,
    mode: AccessMode,
    http: &'a reqwest::Client,
}

pub(super) async fn resolve_token(
    pool: &PgPool,
    user: &UserId,
    provider: Provider,
    probe: bool,
    report: &mut VerificationReport,
) -> AdminResult<String> {
    TokenResolver::new(pool, user, &transport::client()?)
        .resolve(provider, probe, report)
        .await
}

impl<'a> TokenResolver<'a> {
    pub const fn new(pool: &'a PgPool, user: &'a UserId, http: &'a reqwest::Client) -> Self {
        Self { pool, user, http }
    }

    pub async fn resolve(
        &self,
        provider: Provider,
        probe: bool,
        report: &mut VerificationReport,
    ) -> AdminResult<String> {
        if !provider.configured() {
            return Err(AdminError::Unavailable("Connector not configured".into()));
        }
        let span = tracing::info_span!("connector_oauth", user_id = %self.user, provider = provider.slug(), trace_id = %uuid::Uuid::new_v4());
        self.resolve_inner(provider, probe, report)
            .instrument(span)
            .await
    }

    async fn resolve_inner(
        &self,
        provider: Provider,
        probe: bool,
        report: &mut VerificationReport,
    ) -> AdminResult<String> {
        let Self { pool, user, http } = *self;
        let mut tx = pool.begin().await?;
        let mut account = accounts::get_locked_account(&mut tx, user, provider.slug()).await?;
        if account.error_code.as_deref() == Some(FailureKind::RefreshUncertain.code()) {
            return Err(FlowFailure::new(FailureKind::RefreshUncertain).error);
        }
        let row = repo::lock(&mut tx, user, provider.slug()).await?;
        let mode = AccessMode {
            probe,
            preauthorize: row.is_none() && provider.is_salesforce() && account.generation == 0,
        };
        require_connection(&account.status, mode.preauthorize)?;
        let outage_hold = in_outage_hold(&account, row.as_ref().map(|r| r.updated_at));
        mode.admission(outage_hold, account.verified_at.is_some())
            .require()?;
        let resolution = Resolution {
            pool,
            user,
            provider: &provider,
            mode,
            http,
        };
        let (tx, grant) = resolution
            .load_grant(tx, row.map(|r| r.grant), &mut account)
            .await?;
        resolution.complete(tx, &mut account, grant, report).await
    }
}

impl Resolution<'_> {
    async fn load_grant(
        &self,
        mut tx: Tx,
        sealed: Option<EncryptedGrant>,
        account: &mut ProviderConnection,
    ) -> AdminResult<(Tx, Grant)> {
        let Some(sealed) = sealed else {
            let grant = self.mint_first_grant(account.generation).await?;
            return Ok((tx, grant));
        };
        match open_or_flag(&mut tx, self.user, self.provider, &sealed, account).await {
            Ok(grant) => Ok((tx, grant)),
            Err(error) => {
                tx.commit().await?;
                Err(error)
            },
        }
    }

    async fn mint_first_grant(&self, generation: i64) -> AdminResult<Grant> {
        if !self.mode.preauthorize {
            return Err(AdminError::NotFound(
                "Connect your provider account in Systemprompt".into(),
            ));
        }
        salesforce::mint(self.pool, self.user, self.provider, generation).await
    }

    async fn complete(
        &self,
        mut tx: Tx,
        account: &mut ProviderConnection,
        mut grant: Grant,
        report: &mut VerificationReport,
    ) -> AdminResult<String> {
        let reminted = needs_remint(&grant, Utc::now().timestamp());
        if reminted {
            grant =
                salesforce::mint(self.pool, self.user, self.provider, account.generation).await?;
        }
        let plan = TokenPlan::for_grant(&grant, self.mode, reminted, Utc::now().timestamp());
        if let Err(error) = refresh_and_verify(&mut grant, plan, report, self.http).await {
            if report
                .steps
                .iter()
                .any(|step| step.step != "token" && !step.ok)
            {
                account.verified_at = None;
            }
            record_grant_failure(&mut tx, self.user, account, &grant, &error).await?;
            tx.commit().await?;
            return Err(error.error);
        }
        self.persist(&mut tx, account, &grant, plan).await?;
        tx.commit().await?;
        Ok(format!(
            "{} {}",
            grant.authorization_scheme, grant.access_token
        ))
    }

    async fn persist(
        &self,
        tx: &mut Tx,
        account: &mut ProviderConnection,
        grant: &Grant,
        plan: TokenPlan,
    ) -> AdminResult<()> {
        if plan.reseals() {
            repo::store(tx, self.user, self.provider.slug(), &seal(grant)?).await?;
        }
        if plan.verify {
            apply_verification(account, grant);
            accounts::update_account(tx, self.user, account).await?;
        }
        Ok(())
    }
}

async fn refresh_and_verify(
    grant: &mut Grant,
    plan: TokenPlan,
    report: &mut VerificationReport,
    http: &reqwest::Client,
) -> FlowResult<()> {
    let started = std::time::Instant::now();
    let outcome = if plan.refresh {
        super::tokens::refresh_classified(grant, http)
            .await
            .map(|()| "Access token refreshed".to_owned())
    } else {
        Ok(token_state(grant, plan.reminted))
    };
    match outcome {
        Ok(detail) => report.record("token", started, &Ok(detail)),
        Err(failure) => {
            report.record("token", started, &Err(FlowFailure::new(failure.kind).error));
            return Err(failure);
        },
    }
    if plan.verify {
        verify::verify_reporting(grant, http, report)
            .await
            .map_err(FlowFailure::verification)?;
    }
    Ok(())
}
