//! The derived-files stage of `publish_pipeline`: sitemap, llms.txt,
//! robots.txt and the RSS feed, each recorded as its own step. A feed run with
//! no `RssFeedProvider` registered is a skip, not a failure.

use systemprompt::config::AppPaths;
use systemprompt::database::DbPool;
use systemprompt::generator::generate_feed;
use systemprompt::traits::{Job, JobContext};

use super::{PipelineStats, PublishPipelineJob};
use crate::{LlmsTxtGenerationJob, RobotsTxtGenerationJob, SitemapGenerationJob};

impl PublishPipelineJob {
    async fn run_sitemap(&self, ctx: &JobContext, stats: &mut PipelineStats) {
        match SitemapGenerationJob.execute(ctx).await {
            Ok(_result) => {
                tracing::debug!("Sitemap generation completed");
                stats.record_success();
            },
            Err(e) => {
                tracing::error!(error = %e, "Sitemap generation failed");
                stats.record_failure();
            },
        }
    }

    async fn run_llms_txt(&self, ctx: &JobContext, stats: &mut PipelineStats) {
        match LlmsTxtGenerationJob.execute(ctx).await {
            Ok(_result) => {
                tracing::debug!("llms.txt generation completed");
                stats.record_success();
            },
            Err(e) => {
                tracing::error!(error = %e, "llms.txt generation failed");
                stats.record_failure();
            },
        }
    }

    async fn run_feed(&self, paths: &AppPaths, db_pool: &DbPool, stats: &mut PipelineStats) {
        let content_repo = systemprompt::content::ContentRepository::new(db_pool);
        let outcome = generate_feed(content_repo, paths).await;
        record_feed_outcome(outcome, stats);
    }

    async fn run_robots_txt(&self, ctx: &JobContext, stats: &mut PipelineStats) {
        match RobotsTxtGenerationJob.execute(ctx).await {
            Ok(_result) => {
                tracing::debug!("robots.txt generation completed");
                stats.record_success();
            },
            Err(e) => {
                tracing::error!(error = %e, "robots.txt generation failed");
                stats.record_failure();
            },
        }
    }

    pub(super) async fn run_derived_files(
        &self,
        ctx: &JobContext,
        paths: &AppPaths,
        db_pool: &DbPool,
        stats: &mut PipelineStats,
    ) {
        self.run_sitemap(ctx, stats).await;
        self.run_llms_txt(ctx, stats).await;
        self.run_robots_txt(ctx, stats).await;
        self.run_feed(paths, db_pool, stats).await;
    }
}

fn record_feed_outcome<E: std::fmt::Display>(outcome: Result<(), E>, stats: &mut PipelineStats) {
    match outcome {
        Ok(()) => {
            tracing::debug!("RSS feed generation completed");
            stats.record_success();
        },
        Err(e) if e.to_string().contains("No RSS feeds generated") => {
            tracing::debug!("RSS feed generation skipped: no RssFeedProvider registered");
            stats.record_success();
        },
        Err(e) => {
            tracing::error!(error = %e, "RSS feed generation failed");
            stats.record_failure();
        },
    }
}
