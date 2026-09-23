//! Raw evidence expiry and aggregate compaction share one drained-barrier
//! transaction.
//!
//! The barrier (`prepare_reporting_privacy`) delivers pending reporting facts
//! itself, but a backlog above its cap or a claim in flight is reported as
//! 55000, and a full delivery page as 54000. Both mean "the worker has not
//! caught up": this job drains the queue first so the barrier finds little
//! to do, and retries once when it still says wait, instead of marking the
//! night's run failed with nothing to retry until tomorrow.

use crate::error::JobError;
use sqlx::PgPool;
use systemprompt::analytics::snapshots::FeedbackSnapshotsRepository;
use systemprompt::database::DbPool;
use systemprompt::identifiers::UserId;
use systemprompt::traits::{Job, JobContext, JobResult};

// Why: facts drained per pass before the transaction opens; a queue still
// non-empty after DRAIN_PASSES passes is left to the worker.
const DRAIN_PAGE: usize = 10_000;
const DRAIN_PASSES: usize = 20;

#[derive(Debug, Clone, Copy, Default)]
/// Coordinates all organizational aggregates with global raw-evidence expiry.
pub struct PluginUsageRetentionJob;
impl PluginUsageRetentionJob {
    pub async fn execute_with_pool(
        db: &DbPool,
        pool: &PgPool,
        owner: &UserId,
    ) -> Result<JobResult, JobError> {
        tracing::info!(actor=%owner,"Coordinated feedback retention invoked");
        Self::drain_reporting(db).await?;
        match Self::compact_and_expire(pool).await {
            Err(error) if waits_for_worker(&error) => {
                tracing::warn!(error = %error, "retention barrier asked to wait; draining and retrying once");
                Self::drain_reporting(db).await?;
                Self::compact_and_expire(pool).await
            },
            result => result,
        }
    }

    async fn drain_reporting(db: &DbPool) -> Result<(), JobError> {
        for _ in 0..DRAIN_PASSES {
            let processed = systemprompt::system::reporting::process_pending(db, DRAIN_PAGE)
                .await
                .map_err(|error| JobError::other(error.to_string()))?;
            if processed < DRAIN_PAGE {
                return Ok(());
            }
        }
        Ok(())
    }

    async fn compact_and_expire(pool: &PgPool) -> Result<JobResult, JobError> {
        let now = chrono::Utc::now();
        let mut tx = pool.begin().await?;
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

// Why: the barrier's 55000 ("... waits for ...") and 54000 ("... bound
// exceeded") are its two "wait for the worker" answers; the error arrives
// stringified through `JobError`, so the message is matched in text.
fn waits_for_worker(error: &JobError) -> bool {
    let text = error.to_string();
    text.contains("waits for") || text.contains("bound exceeded")
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
        let pool = db.write_pool();
        Ok(Self::execute_with_pool(db, &pool, &ctx.actor().user_id).await?)
    }
}
systemprompt::traits::submit_job!(&PluginUsageRetentionJob);
