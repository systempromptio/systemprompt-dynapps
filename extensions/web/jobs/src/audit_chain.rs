//! `audit_chain_verify` job: checks the audit hash chain nightly.
//!
//! One call to `record_audit_chain_verification` (schema
//! `51_audit_integrity.sql`), which re-hashes the last two days of the
//! `ai_requests` and `ai_safety_findings` chains, writes one
//! `audit_chain_verifications` row per table with the anchor and head it saw,
//! and drops links older than every retention floor. A run that finds a gap,
//! a broken link, an altered or deleted row, or an unlinked row fails, so the
//! break surfaces in the scheduler's run history and `infra logs`. The run
//! message also carries `unsealed_rows`: rows `audit_chain_seal` has not
//! linked yet, which is the seal's lag rather than a break.

use crate::error::JobError;
use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

const WINDOW_DAYS: i32 = 2;

#[derive(Debug, Clone, Copy, Default)]
pub struct AuditChainVerifyJob;

impl AuditChainVerifyJob {
    pub async fn execute_with_pool(pool: &PgPool) -> Result<JobResult, JobError> {
        let problems = sqlx::query_scalar!(
            r#"SELECT record_audit_chain_verification(make_interval(days => $1)) AS "problems!""#,
            WINDOW_DAYS
        )
        .fetch_one(pool)
        .await?;
        let unsealed_rows = sqlx::query_scalar!(
            r#"SELECT audit_chain_unsealed('ai_requests') + audit_chain_unsealed('ai_safety_findings') AS "unsealed_rows!""#
        )
        .fetch_one(pool)
        .await?;
        if problems > 0 {
            return Err(JobError::other(format!(
                "audit chain verification found {problems} problems (unsealed_rows={unsealed_rows}); see audit_chain_verifications"
            )));
        }
        Ok(JobResult::success()
            .with_message(format!("audit chain intact; unsealed_rows={unsealed_rows}")))
    }
}

#[async_trait::async_trait]
impl Job for AuditChainVerifyJob {
    fn name(&self) -> &'static str {
        "audit_chain_verify"
    }
    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }
    fn description(&self) -> &'static str {
        "Re-hashes the last two days of the ai_requests and ai_safety_findings audit chains and records any break"
    }
    fn schedule(&self) -> &'static str {
        "0 45 4 * * *"
    }
    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        tracing::info!(actor = %ctx.actor().user_id, "Audit chain verification invoked");
        let db = ctx.get::<DbPool>()?;
        Ok(Self::execute_with_pool(&db.write_pool()).await?)
    }
}
systemprompt::traits::submit_job!(&AuditChainVerifyJob);
