//! `audit_chain_seal` job: links new audit rows into the hash chain.
//!
//! One call to `audit_chain_seal` (schema `51_audit_integrity.sql`) every ten
//! seconds. It takes a transaction-scoped advisory lock, so exactly one
//! replica seals at a time, links up to `BATCH` new `ai_requests` and
//! `ai_safety_findings` rows each past `audit_chain_cursors`, and advances the
//! cursors, all in the one statement's transaction. The gateway's inserts no
//! longer touch `audit_chain_heads`, so the chain is eventually consistent
//! (within one interval) while the tamper guards stay synchronous. Returns 0
//! when the chain is off and -1 when another replica holds the lock.

use crate::error::JobError;
use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

const BATCH: i32 = 5000;

#[derive(Debug, Clone, Copy, Default)]
pub struct AuditChainSealJob;

impl AuditChainSealJob {
    pub async fn execute_with_pool(pool: &PgPool) -> Result<JobResult, JobError> {
        let sealed = sqlx::query_scalar!(r#"SELECT audit_chain_seal($1) AS "sealed!""#, BATCH)
            .fetch_one(pool)
            .await?;
        if sealed < 0 {
            return Ok(JobResult::success()
                .with_message("another replica is sealing the audit chain".to_owned()));
        }
        let linked = u64::try_from(sealed).unwrap_or(0);
        Ok(JobResult::success()
            .with_stats(linked, 0)
            .with_message(format!("sealed {sealed} audit rows")))
    }
}

#[async_trait::async_trait]
impl Job for AuditChainSealJob {
    fn name(&self) -> &'static str {
        "audit_chain_seal"
    }
    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }
    fn description(&self) -> &'static str {
        "Links new ai_requests and ai_safety_findings rows into the audit hash chain"
    }
    fn schedule(&self) -> &'static str {
        "*/10 * * * * *"
    }
    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        let db = ctx.get::<DbPool>()?;
        Ok(Self::execute_with_pool(&db.write_pool()).await?)
    }
}
systemprompt::traits::submit_job!(&AuditChainSealJob);
