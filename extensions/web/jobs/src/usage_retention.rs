//! `plugin_usage_retention` job: expires raw evidence older than 90 days.
//!
//! Calls `expire_raw_evidence` (schema `32_raw_retention.sql`) in batches of
//! 5,000 rows per table, each call its own transaction, until it
//! removes nothing or the run's time budget is spent; whatever the budget
//! leaves is taken by the next night's run. The per-conversation record and
//! the daily rollups are kept, so every figure the console shows outlives the
//! raw rows.

use crate::error::JobError;
use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

const RAW_RETENTION_DAYS: i64 = 90;

const BATCH_SIZE: i32 = 5_000;

const RUN_BUDGET: std::time::Duration = std::time::Duration::from_mins(15);

#[derive(Debug, Clone, Copy, Default)]
pub struct PluginUsageRetentionJob;

impl PluginUsageRetentionJob {
    pub async fn execute_with_pool(pool: &PgPool) -> Result<JobResult, JobError> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(RAW_RETENTION_DAYS);
        let started = std::time::Instant::now();
        let mut deleted: u64 = 0;
        let mut batches: u64 = 0;
        loop {
            let removed = sqlx::query_scalar!(
                r#"SELECT expire_raw_evidence($1, $2) AS "removed!""#,
                cutoff,
                BATCH_SIZE
            )
            .fetch_one(pool)
            .await?;
            let removed =
                u64::try_from(removed).map_err(|error| JobError::other(error.to_string()))?;
            batches += 1;
            deleted += removed;
            if removed == 0 {
                return Ok(JobResult::success()
                    .with_stats(deleted, 0)
                    .with_message(format!("{deleted} row(s) expired in {batches} batch(es)")));
            }
            if started.elapsed() >= RUN_BUDGET {
                tracing::warn!(
                    deleted,
                    batches,
                    "Raw evidence retention budget spent; the next run continues"
                );
                return Ok(JobResult::success()
                    .with_stats(deleted, 0)
                    .with_message(format!(
                        "{deleted} row(s) expired in {batches} batch(es); budget spent, more remain"
                    )));
            }
        }
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
        "Expires hook events, gateway requests and their evidence older than 90 days"
    }
    fn schedule(&self) -> &'static str {
        "0 20 3 * * *"
    }
    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        tracing::info!(actor = %ctx.actor().user_id, "Raw evidence retention invoked");
        let db = ctx.get::<DbPool>()?;
        Ok(Self::execute_with_pool(&db.write_pool()).await?)
    }
}
systemprompt::traits::submit_job!(&PluginUsageRetentionJob);
