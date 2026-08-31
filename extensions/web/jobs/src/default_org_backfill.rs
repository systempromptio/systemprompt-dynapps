//! Backfill: give every user without an organization the default one.
//!
//! Membership is the spine of the admin surface. Every analytics query, the
//! customer usage report, and every plan grant scope through
//! `organization_members`, so a user with no row there is not merely
//! unentitled — they are invisible to the dashboards meant to account for
//! them, and the pages report zero rather than reporting a gap.
//!
//! Until the `is_default` organization existed, nothing filled that row for an
//! ordinary user: SSO just-in-time provisioning matched an email domain and
//! left an unmatched arrival unattached, and only the single configured system
//! admin was joined to anything (see [`crate::PlatformAdminBootstrapJob`]).
//! Provisioning now falls back to the default organization, which fixes every
//! future arrival; this job is what fixes the ones already here.
//!
//! Runs once at boot as a `scheduler.bootstrap_jobs` entry (`schedule()` is
//! empty), after the plan loader has upserted the organizations. It is
//! idempotent and strictly additive: it only ever inserts for a user who has no
//! membership at all, and never moves or re-roles a user who has one —
//! membership is the billing and authorization boundary, so relocating someone
//! is an operator decision, not a boot side effect.

use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::error::JobError;
use systemprompt_web_admin::repositories::organizations::crud as org_repo;
use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultOrgBackfillJob;

#[async_trait::async_trait]
impl Job for DefaultOrgBackfillJob {
    fn name(&self) -> &'static str {
        "default_org_backfill"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn description(&self) -> &'static str {
        "Join every user without an organization to the default organization, so usage \
         reporting and plan entitlements can account for them"
    }

    fn schedule(&self) -> &'static str {
        ""
    }

    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        Ok(execute_inner(ctx).await?)
    }
}

async fn execute_inner(ctx: &JobContext) -> Result<JobResult, JobError> {
    let start = std::time::Instant::now();

    let db_pool = ctx.db_pool::<DbPool>().ok_or(MarketplaceError::Internal(
        "Database not available in job context".to_owned(),
    ))?;
    let pool = db_pool.write_pool().ok_or(MarketplaceError::Internal(
        "PgPool not available from database".to_owned(),
    ))?;

    let outcome = backfill_default_organization(&pool).await?;

    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    tracing::info!(
        joined = outcome.joined,
        "default-organization backfill completed"
    );
    Ok(JobResult::success().with_duration(duration_ms))
}

/// How many users the backfill attached; one boot log line renders it.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BackfillOutcome {
    pub joined: u64,
}

// Why: a single INSERT … SELECT rather than a read-then-write loop, so a user
// created between the two halves cannot be missed, and a concurrent boot on
// another instance cannot double-insert. `ON CONFLICT DO NOTHING` covers the
// race on the `organization_members` primary key; `NOT EXISTS` is what makes
// the job additive rather than a reassignment.
#[doc(hidden)]
pub async fn backfill_default_organization(
    pool: &sqlx::PgPool,
) -> Result<BackfillOutcome, JobError> {
    let Some(org_id) = org_repo::find_default_organization(pool).await? else {
        tracing::warn!(
            "default-organization backfill: no active is_default organization exists; \
             set `default: true` on one organization in services/access-control/plans.yaml"
        );
        return Ok(BackfillOutcome::default());
    };

    let joined = sqlx::query!(
        "INSERT INTO organization_members (user_id, org_id, org_role)
         SELECT u.id, $1, 'member'
           FROM users u
          WHERE NOT EXISTS (
                SELECT 1 FROM organization_members m WHERE m.user_id = u.id
              )
         ON CONFLICT (user_id) DO NOTHING",
        org_id,
    )
    .execute(pool)
    .await
    .map_err(MarketplaceError::from)?
    .rows_affected();

    if joined > 0 {
        tracing::info!(
            joined,
            org_id = %org_id,
            "default-organization backfill: users joined the default organization"
        );
    }
    Ok(BackfillOutcome { joined })
}

systemprompt::traits::submit_job!(&DefaultOrgBackfillJob);
