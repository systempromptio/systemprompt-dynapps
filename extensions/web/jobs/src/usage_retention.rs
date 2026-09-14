//! Raw evidence expiry and aggregate compaction share one drained-barrier
//! transaction.

use crate::error::JobError;
use sqlx::PgPool;
use systemprompt::analytics::snapshots::FeedbackSnapshotsRepository;
use systemprompt::database::DbPool;
use systemprompt::identifiers::UserId;
use systemprompt::traits::{Job, JobContext, JobResult};

#[derive(Debug, Clone, Copy, Default)]
/// Coordinates all organizational aggregates with global raw-evidence expiry.
pub struct PluginUsageRetentionJob;
impl PluginUsageRetentionJob {
    pub async fn execute_with_pool(pool: &PgPool, owner: &UserId) -> Result<JobResult, JobError> {
        let now = chrono::Utc::now();
        let mut tx = pool.begin().await?;
        tracing::info!(actor=%owner,"Coordinated feedback retention invoked");
        let compacted = FeedbackSnapshotsRepository::compact_all_in(&mut tx, now)
            .await
            .map_err(|error| JobError::other(error.to_string()))?;
        let cutoff = now - chrono::Duration::days(90);
        let raw = sqlx::query_scalar!(r#"SELECT expire_feedback_raw($1) AS "deleted!""#, cutoff)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        let deleted = u64::try_from(raw)
            .map_err(|error| JobError::other(error.to_string()))?
            .saturating_add(compacted.removed_facts);
        Ok(JobResult::success().with_stats(deleted, 0))
    }
}
#[async_trait::async_trait]
impl Job for PluginUsageRetentionJob {
    fn name(&self) -> &'static str {
        "plugin_usage_retention"
    }
    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }
    fn description(&self) -> &'static str {
        "Expires raw evidence after drained snapshot checkpoints and privacy correction barriers"
    }
    fn schedule(&self) -> &'static str {
        "0 20 3 * * *"
    }
    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        let db = ctx
            .db_pool::<DbPool>()
            .ok_or(JobError::MissingContext("DbPool"))?;
        let pool = db
            .write_pool()
            .ok_or(JobError::MissingContext("write PgPool"))?;
        Ok(Self::execute_with_pool(&pool, &ctx.actor().user_id).await?)
    }
}
systemprompt::traits::submit_job!(&PluginUsageRetentionJob);
