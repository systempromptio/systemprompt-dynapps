//! The `dynbrain_index` job: runs the indexer under a lease.
//!
//! The scheduler already runs a cluster job on one replica at a time; the
//! lease (`ctx.leases()`) also covers a run that outlives its tick and a
//! manual run started while a scheduled one is working. A replica that does
//! not get the lease reports it and leaves the queue to the one that has it.
//!
//! The models are the catalog's: `embedding_provider` / `embedding_model`
//! (`OpenAI` `text-embedding-3-large`, 1536 dimensions) and `synopsis_provider`
//! / `synopsis_model` (Anthropic `claude-haiku-5-5`), and `context_strategy`
//! picks how chunks are situated. All can be set as parameters of the
//! scheduler entry. The synopsis requests are audited as this job's actor,
//! the entry's `owner`. The job does nothing when the instance has no AI
//! service, and an embedding endpoint without its key leaves the chunks
//! queued.

use std::sync::Arc;
use std::time::Duration;

use systemprompt::database::DbPool;
use systemprompt::extension::ExtensionContext;
use systemprompt::identifiers::{Actor, LeaseKey};
use systemprompt::system::AppContext;
use systemprompt::traits::{
    CoordinationError, Job, JobContext, JobResult, MissingDependency, ProviderError,
};

use super::context::ContextStrategy;
use super::core_ai::{CoreEmbedder, CoreSummarizer};
use super::index::Indexer;
use super::run::IndexSettings;
use crate::error::BrainError;
use crate::repository::BrainRepository;

pub const JOB_NAME: &str = "dynbrain_index";
const LEASE_TTL: Duration = Duration::from_mins(15);

#[derive(Debug, thiserror::Error)]
pub enum IndexJobError {
    #[error(transparent)]
    Dependency(#[from] MissingDependency),
    #[error("the Brain store failed")]
    Store(#[from] BrainError),
    #[error("the index lease could not be taken")]
    Lease(#[from] CoordinationError),
}

impl From<IndexJobError> for ProviderError {
    fn from(error: IndexJobError) -> Self {
        Self::Internal(Box::new(error))
    }
}

struct Models {
    embedding_provider: String,
    embedding_model: String,
    synopsis_provider: String,
    synopsis_model: String,
    strategy: ContextStrategy,
}

impl Models {
    fn from_context(ctx: &JobContext) -> Self {
        let text = |key: &str, default: &str| {
            ctx.get_parameter(key)
                .map_or_else(|| default.to_owned(), Clone::clone)
        };
        let strategy = text("context_strategy", "auto");
        Self {
            embedding_provider: text("embedding_provider", "openai"),
            embedding_model: text("embedding_model", "text-embedding-3-large"),
            synopsis_provider: text("synopsis_provider", "anthropic"),
            synopsis_model: text("synopsis_model", "claude-haiku-5-5"),
            strategy: ContextStrategy::parse(&strategy).unwrap_or_else(|| {
                tracing::warn!(%strategy, "dynbrain: unknown context_strategy, using auto");
                ContextStrategy::Auto
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DynbrainIndexJob;

impl DynbrainIndexJob {
    async fn run(ctx: &JobContext) -> Result<JobResult, IndexJobError> {
        let app = ctx.get::<Arc<AppContext>>()?;
        let Some(ai) = app.ai_service_arc() else {
            return Ok(
                JobResult::success().with_message("no AI provider is configured; indexing is idle")
            );
        };
        let leases = ctx.get::<Arc<dyn ExtensionContext>>()?.leases();
        let key = LeaseKey::new(JOB_NAME);
        if leases.claim(&key, LEASE_TTL).await?.is_none() {
            return Ok(JobResult::success().with_message("another replica is indexing"));
        }
        let models = Models::from_context(ctx);
        let actor = Actor::job(ctx.actor().user_id.clone(), JOB_NAME);
        let indexer = Indexer::new(
            BrainRepository::new(ctx.get::<DbPool>()?.write_pool()),
            Arc::new(CoreEmbedder::new(
                Arc::clone(&ai),
                models.embedding_provider,
                models.embedding_model,
            )),
            Some(Arc::new(CoreSummarizer::new(
                ai,
                actor,
                models.synopsis_provider,
                models.synopsis_model,
            ))),
            IndexSettings {
                strategy: models.strategy,
                ..IndexSettings::default()
            },
        );
        let outcome = indexer.run().await;
        if let Err(error) = leases.release(&key).await {
            tracing::warn!(%error, "dynbrain: the index lease was not released");
        }
        let report = outcome?;
        let processed = report.indexed + report.purged + report.embedded;
        Ok(JobResult::success()
            .with_stats(processed as u64, report.failed as u64)
            .with_message(report.summary()))
    }
}

#[async_trait::async_trait]
impl Job for DynbrainIndexJob {
    fn name(&self) -> &'static str {
        JOB_NAME
    }

    fn description(&self) -> &'static str {
        "Chunks, situates and embeds the articles of the native Brain that were written"
    }

    fn schedule(&self) -> &'static str {
        "0 */2 * * * *"
    }

    async fn execute(&self, ctx: &JobContext) -> Result<JobResult, ProviderError> {
        tracing::info!(actor = %ctx.actor().user_id, "Brain index invoked");
        Ok(Self::run(ctx).await?)
    }
}

systemprompt::traits::submit_job!(&DynbrainIndexJob);
