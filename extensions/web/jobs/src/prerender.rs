//! `site_content_prerender` job: renders ingested content into static page
//! artifacts. Named apart from core's own `content_prerender` so the two
//! never collide in the scheduler.

use std::sync::Arc;
use systemprompt::analytics::ContentAnalyticsRepository;
use systemprompt::config::AppPaths;
use systemprompt::database::DbPool;
use systemprompt::generator::prerender_content;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::error::JobError;

#[derive(Debug, Clone, Copy, Default)]
pub struct SiteContentPrerenderJob;

#[async_trait::async_trait]
impl Job for SiteContentPrerenderJob {
    fn name(&self) -> &'static str {
        "site_content_prerender"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn description(&self) -> &'static str {
        "Prerenders static HTML pages with enriched content from database"
    }

    fn schedule(&self) -> &'static str {
        "0 */15 * * * *"
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

    tracing::debug!("Content prerender started");

    let db_pool = ctx.get::<DbPool>()?;
    let paths = ctx.get::<Arc<AppPaths>>()?.as_ref();

    let content_repo = systemprompt::content::ContentRepository::new(db_pool);

    let content_analytics = ContentAnalyticsRepository::new(db_pool);

    prerender_content(
        DbPool::clone(db_pool),
        content_repo,
        content_analytics,
        paths,
    )
    .await?;

    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

    tracing::debug!(duration_ms, "Content prerender completed");

    Ok(JobResult::success().with_duration(duration_ms))
}

systemprompt::traits::submit_job!(&SiteContentPrerenderJob);
