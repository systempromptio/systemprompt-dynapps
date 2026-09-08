//! Daily Drive discovery requests under the scheduler's configured owner.

use crate::error::JobError;
use systemprompt::config::SecretsBootstrap;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult, ProviderError};
use systemprompt_mcp_shared::knowledge::config::KnowledgeBankConfig;
use systemprompt_mcp_shared::knowledge::drive::store;

#[derive(Debug, Clone, Copy, Default)]
pub struct KnowledgeDriveSyncJob;

#[async_trait::async_trait]
impl Job for KnowledgeDriveSyncJob {
    fn name(&self) -> &'static str {
        "knowledge_drive_sync"
    }
    fn description(&self) -> &'static str {
        "Queue service-account Google Drive synchronization"
    }
    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }
    fn schedule(&self) -> &'static str {
        "0 0 3 * * *"
    }
    async fn execute(&self, ctx: &JobContext) -> Result<JobResult, ProviderError> {
        execute(ctx).await.map_err(Into::into)
    }
}

async fn execute(ctx: &JobContext) -> Result<JobResult, JobError> {
    let config = KnowledgeBankConfig::load(&KnowledgeBankConfig::resolve_path())
        .map_err(|e| JobError::Config(e.to_string()))?;
    if !config.google_drive.enabled {
        return Ok(JobResult::success().with_stats(0, 0));
    }
    let secrets = SecretsBootstrap::require().map_err(|e| JobError::Config(e.to_string()))?;
    let db = ctx
        .db_pool::<DbPool>()
        .ok_or(JobError::MissingContext("DbPool"))?;
    let separate_pool = if let Some(url) = secrets.get(&config.knowledge_database.secret) {
        Some(
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(2)
                .connect(url)
                .await?,
        )
    } else {
        None
    };
    let instance_pool = db.pool_arc().map_err(|e| JobError::Config(e.to_string()))?;
    let pool = separate_pool.as_ref().unwrap_or(&instance_pool);
    let mut count = 0;
    for (id, source) in &config.google_drive.sources {
        store::enqueue(
            pool,
            id,
            &source.drive_id,
            ctx.actor().user_id.as_str(),
            store::SyncRequest {
                force: false,
                rebuild: false,
            },
        )
        .await
        .map_err(|e| JobError::Other(e.to_string()))?;
        count += 1;
    }
    tracing::info!(actor = %ctx.actor().user_id, sources = count, "Daily Drive sync queued");
    if let Some(pool) = separate_pool {
        pool.close().await;
    }
    Ok(JobResult::success().with_stats(count, 0))
}

systemprompt::traits::submit_job!(&KnowledgeDriveSyncJob);
