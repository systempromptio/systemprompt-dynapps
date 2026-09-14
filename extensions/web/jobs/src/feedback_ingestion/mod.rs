//! Durable capture-to-facts adapter. Leases and producer barriers share
//! database transactions.
mod backfill;
mod repository;

use crate::JobError;
use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::identifiers::UserId;
use systemprompt::traits::{Job, JobContext, JobResult};

pub use repository::{FeedbackIngestionRepository, IngestionLease};

/// Scheduled bounded transfer of committed source revisions into normalized
/// facts.
#[derive(Debug, Clone, Copy, Default)]
pub struct FeedbackIngestionJob;

impl FeedbackIngestionJob {
    pub async fn execute_with_pool(pool: &PgPool, owner: &UserId) -> Result<JobResult, JobError> {
        let repository = FeedbackIngestionRepository::new(pool.clone());
        repository.backfill_page(128).await?;
        let mut completed = 0;
        for lease in repository.claim(64).await? {
            match repository.process(owner, &lease).await {
                Ok(true) => completed += 1,
                Ok(false) => {},
                Err(error) => {
                    repository.retry(&lease).await?;
                    tracing::warn!(%error, "Feedback ingestion batch will retry");
                },
            }
        }
        Ok(JobResult::success().with_stats(completed, 0))
    }
}

#[async_trait::async_trait]
impl Job for FeedbackIngestionJob {
    fn name(&self) -> &'static str {
        "feedback_ingestion"
    }
    fn description(&self) -> &'static str {
        "Projects committed authenticated evidence and corrections into normalized feedback facts"
    }
    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }
    fn schedule(&self) -> &'static str {
        "*/5 * * * * *"
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
systemprompt::traits::submit_job!(&FeedbackIngestionJob);
