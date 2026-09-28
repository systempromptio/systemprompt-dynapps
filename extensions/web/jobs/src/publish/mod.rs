//! `publish_pipeline` job: the ordered composite every other job runs under.
//!
//! Order is load-bearing — bundles must exist before assets are copied, and
//! content must be ingested before it can be prerendered. A failing sub-job is
//! recorded and the pipeline continues, so one broken stage does not block the
//! rest.

use std::sync::Arc;

use systemprompt::config::AppPaths;
use systemprompt::content::ContentRepository;
use systemprompt::database::DbPool;
use systemprompt::generator::{organize_dist_assets, prerender_pages};
use systemprompt::traits::{Job, JobContext, JobResult};

use super::{
    BundleAdminCssJob, ContentIngestionJob, CopyExtensionAssetsJob, SiteContentPrerenderJob,
};
use crate::error::JobError;
use systemprompt_web_shared::error::MarketplaceError;

mod derived;

#[doc(hidden)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PipelineStats {
    pub succeeded: u64,
    pub failed: u64,
}

impl PipelineStats {
    pub const fn record_success(&mut self) {
        self.succeeded += 1;
    }

    pub const fn record_failure(&mut self) {
        self.failed += 1;
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PublishPipelineJob;

impl PublishPipelineJob {
    async fn run_ingestion(&self, ctx: &JobContext, stats: &mut PipelineStats) {
        match ContentIngestionJob.execute(ctx).await {
            Ok(result) => {
                tracing::debug!(
                    processed = result.items_processed.unwrap_or(0),
                    errors = result.items_failed.unwrap_or(0),
                    "Content ingestion completed"
                );
                stats.record_success();
            },
            Err(e) => {
                tracing::error!(error = %e, "Content ingestion failed");
                stats.record_failure();
            },
        }
    }

    async fn run_bundle_admin_css(&self, paths: &AppPaths, stats: &mut PipelineStats) {
        match BundleAdminCssJob::execute_bundle(paths).await {
            Ok(result) => {
                tracing::debug!(
                    bundled = result.items_processed.unwrap_or(0),
                    "Admin CSS bundle completed"
                );
                stats.record_success();
            },
            Err(e) => {
                tracing::error!(error = %e, "Admin CSS bundle failed");
                stats.record_failure();
            },
        }
    }

    async fn run_asset_copy(&self, paths: &AppPaths, stats: &mut PipelineStats) {
        match CopyExtensionAssetsJob::execute_copy(paths).await {
            Ok(result) => {
                tracing::debug!(
                    copied = result.items_processed.unwrap_or(0),
                    failed = result.items_failed.unwrap_or(0),
                    "Asset copy completed"
                );
                stats.record_success();
            },
            Err(e) => {
                tracing::error!(error = %e, "Asset copy failed");
                stats.record_failure();
            },
        }
    }

    async fn run_prerender(&self, ctx: &JobContext, stats: &mut PipelineStats) {
        match SiteContentPrerenderJob.execute(ctx).await {
            Ok(_result) => {
                tracing::debug!("Content prerender completed");
                stats.record_success();
            },
            Err(e) => {
                tracing::error!(error = %e, "Content prerender failed");
                stats.record_failure();
            },
        }
    }

    async fn run_page_prerender(
        &self,
        paths: &AppPaths,
        db_pool: &DbPool,
        stats: &mut PipelineStats,
    ) {
        let Some(content_repo) = open_content_repo(db_pool, stats) else {
            return;
        };
        match prerender_pages(Arc::clone(db_pool), content_repo, paths).await {
            Ok(results) => {
                tracing::debug!(page_count = results.len(), "Page prerendering completed");
                stats.record_success();
            },
            Err(e) => {
                tracing::error!(error = %e, "Page prerendering failed");
                stats.record_failure();
            },
        }
    }

    async fn run_asset_organization(&self, paths: &AppPaths, stats: &mut PipelineStats) {
        let dist_dir = paths.web().dist().to_path_buf();
        match organize_dist_assets(&dist_dir).await {
            Ok((css_count, js_count)) => {
                tracing::debug!(css = css_count, js = js_count, "Assets organized");
                stats.record_success();
            },
            Err(e) => {
                tracing::error!(error = %e, "Asset organization failed");
                stats.record_failure();
            },
        }
    }
}

#[async_trait::async_trait]
impl Job for PublishPipelineJob {
    fn name(&self) -> &'static str {
        "publish_pipeline"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn description(&self) -> &'static str {
        "Full content publishing pipeline: ingestion, assets, prerender, pages, sitemap, robots.txt, RSS, llms.txt"
    }

    fn schedule(&self) -> &'static str {
        "0 */15 * * * *"
    }
    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        Ok(self.execute_inner(ctx).await?)
    }
}

impl PublishPipelineJob {
    async fn execute_inner(&self, ctx: &JobContext) -> Result<JobResult, JobError> {
        let start_time = std::time::Instant::now();

        let db_pool = ctx.db_pool::<DbPool>().ok_or(MarketplaceError::Internal(
            "Database not available in job context".to_owned(),
        ))?;
        let paths = ctx
            .app_paths::<Arc<AppPaths>>()
            .ok_or(MarketplaceError::Internal(
                "AppPaths not available in job context".to_owned(),
            ))?
            .as_ref();

        tracing::debug!(actor = %ctx.actor().user_id.as_str(), "Publish pipeline started");

        let mut stats = PipelineStats::default();

        self.run_ingestion(ctx, &mut stats).await;
        self.run_bundle_admin_css(paths, &mut stats).await;
        self.run_asset_copy(paths, &mut stats).await;
        self.run_prerender(ctx, &mut stats).await;
        self.run_page_prerender(paths, db_pool, &mut stats).await;
        self.run_derived_files(ctx, paths, db_pool, &mut stats)
            .await;
        stats.record_success();
        self.run_asset_organization(paths, &mut stats).await;

        let duration_ms = u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);

        tracing::info!(
            steps_succeeded = stats.succeeded,
            steps_failed = stats.failed,
            duration_ms,
            "Publish pipeline completed"
        );

        Ok(JobResult::success()
            .with_stats(stats.succeeded, stats.failed)
            .with_duration(duration_ms))
    }
}

fn open_content_repo(db_pool: &DbPool, stats: &mut PipelineStats) -> Option<ContentRepository> {
    match ContentRepository::new(db_pool) {
        Ok(repo) => Some(repo),
        Err(e) => {
            tracing::error!(error = %e, "Content repository init failed");
            stats.record_failure();
            None
        },
    }
}

systemprompt::traits::submit_job!(&PublishPipelineJob);
