//! `content_ingestion` job: parses the `services/content/` markdown tree into
//! the database.
//!
//! The blog config and `delete_orphans` come from this job's own context: the
//! config is read from the job's `AppPaths`, and the orphan switch is a job
//! parameter rather than an environment variable that would reach every job.
//!
//! Every ingestion is versioned. It runs under an advisory lock and records the
//! release that ingested in `content_ingestion_stamp`; a release older than the
//! recorded one refuses, so a previous-image pod restarting mid-rollout cannot
//! ingest its older tree and delete the newer pages as orphans.
//! `ensure_current` is the site render's variant: it ingests only when no
//! release, or an older one, is recorded, and a replica that finds another
//! already ingesting waits on the lock and then skips, so a fresh node renders
//! the content rather than an empty site.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::config::AppPaths;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::error::JobError;
use systemprompt_web_content::services::IngestionService;
use systemprompt_web_shared::config::BlogConfigValidated;
use systemprompt_web_shared::error::MarketplaceError;
use systemprompt_web_shared::models::IngestionOptions;

#[derive(Debug, Clone, Copy, Default)]
pub struct ContentIngestionJob;

impl ContentIngestionJob {
    pub async fn execute_with_config(
        pool: Arc<PgPool>,
        config: &BlogConfigValidated,
    ) -> Result<JobResult, MarketplaceError> {
        Self::execute_with_options(pool, config, IngestionOptions::default()).await
    }

    pub async fn execute_with_options(
        pool: Arc<PgPool>,
        config: &BlogConfigValidated,
        options: IngestionOptions,
    ) -> Result<JobResult, MarketplaceError> {
        let start = std::time::Instant::now();

        tracing::debug!(
            delete_orphans = options.delete_orphans,
            "Blog content ingestion started"
        );

        let ingestion_service = IngestionService::new(pool);

        let mut total_processed = 0u64;
        let mut total_errors = 0u64;
        let mut total_orphans_deleted = 0u64;

        for source in config.enabled_sources() {
            tracing::debug!(
                source_id = %source.source_id(),
                path = %source.path().display(),
                "Ingesting source"
            );

            let (processed, errors, orphans) =
                ingest_source(&ingestion_service, source, options).await;
            total_processed += processed;
            total_errors += errors;
            total_orphans_deleted += orphans;
        }

        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

        tracing::debug!(
            files_processed = total_processed,
            orphans_deleted = total_orphans_deleted,
            errors = total_errors,
            duration_ms = duration_ms,
            "Blog content ingestion completed"
        );

        Ok(JobResult::success()
            .with_stats(total_processed, total_errors)
            .with_duration(duration_ms))
    }
}

#[async_trait::async_trait]
impl Job for ContentIngestionJob {
    fn name(&self) -> &'static str {
        "blog_content_ingestion"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn description(&self) -> &'static str {
        "Ingests markdown content from configured blog directories into the database. Set the `delete_orphans` job parameter to true to clean up orphaned records."
    }

    fn schedule(&self) -> &'static str {
        "0 0 * * * *"
    }

    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        Ok(versioned(ctx, IngestWhen::Always).await?)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IngestWhen {
    Always,
    IfBehind,
}

const RELEASE: &str = env!("CARGO_PKG_VERSION");

#[must_use]
pub fn release_key(version: &str) -> Vec<u64> {
    version
        .split(['-', '+'])
        .next()
        .unwrap_or_default()
        .split('.')
        .map(|part| part.parse().unwrap_or(0))
        .collect()
}

pub(crate) async fn ensure_current(ctx: &JobContext) -> Result<JobResult, JobError> {
    versioned(ctx, IngestWhen::IfBehind).await
}

async fn versioned(ctx: &JobContext, when: IngestWhen) -> Result<JobResult, JobError> {
    let pool = ctx.get::<DbPool>()?.write_pool();
    let mut conn = pool.acquire().await?;
    // Why: core's scheduler holds pg_advisory_lock(hashtext(<job name>)) for
    // the job's run, so the stamp lock takes a key of its own.
    sqlx::query!("SELECT pg_advisory_lock(hashtext('content_ingestion:stamp'))")
        .execute(&mut *conn)
        .await?;
    let outcome = ingest_under_lock(ctx, &mut conn, when).await;
    if let Err(error) = sqlx::Connection::close(conn.detach()).await {
        tracing::warn!(%error, "content ingestion lock connection did not close cleanly");
    }
    outcome
}

async fn ingest_under_lock(
    ctx: &JobContext,
    conn: &mut sqlx::PgConnection,
    when: IngestWhen,
) -> Result<JobResult, JobError> {
    let recorded = sqlx::query_scalar!("SELECT version FROM content_ingestion_stamp WHERE id")
        .fetch_optional(&mut *conn)
        .await?;
    if let Some(recorded) = &recorded {
        let ours = release_key(RELEASE);
        let theirs = release_key(recorded);
        if theirs > ours {
            tracing::warn!(
                recorded = %recorded,
                release = RELEASE,
                "content ingestion skipped: a newer release already ingested"
            );
            return Ok(JobResult::success()
                .with_message(format!("skipped: release {recorded} already ingested")));
        }
        if when == IngestWhen::IfBehind && theirs == ours {
            return Ok(JobResult::success().with_message("content already current"));
        }
    }
    let result = execute_inner(ctx).await?;
    if result.success {
        sqlx::query!(
            "INSERT INTO content_ingestion_stamp (id, version, ingested_at)
             VALUES (TRUE, $1, CURRENT_TIMESTAMP)
             ON CONFLICT (id) DO UPDATE SET version = EXCLUDED.version,
                                            ingested_at = EXCLUDED.ingested_at",
            RELEASE
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(result)
}

fn job_services_dir(ctx: &JobContext) -> Result<std::path::PathBuf, JobError> {
    Ok(ctx
        .get::<Arc<AppPaths>>()?
        .system()
        .services()
        .to_path_buf())
}

async fn execute_inner(ctx: &JobContext) -> Result<JobResult, JobError> {
    let db = ctx.get::<DbPool>()?;

    let pool = db.write_pool();

    let services_dir = job_services_dir(ctx)?;

    let Some(config) = BlogConfigValidated::load_from_services(&services_dir)? else {
        tracing::debug!(
            "Blog content ingestion skipped: no blog config configured for this profile"
        );
        return Ok(JobResult::success().with_message("skipped: no blog config"));
    };

    let delete_orphans = ctx
        .get_parameter("delete_orphans")
        .is_some_and(|v| v.eq_ignore_ascii_case("true") || v == "1");

    let options = IngestionOptions::default().with_delete_orphans(delete_orphans);

    Ok(ContentIngestionJob::execute_with_options(pool, &config, options).await?)
}

async fn ingest_source(
    ingestion_service: &IngestionService,
    source: &systemprompt_web_shared::config::ContentSourceValidated,
    options: IngestionOptions,
) -> (u64, u64, u64) {
    match ingestion_service
        .ingest_path_with_options(
            source.path(),
            source.source_id(),
            source.category_id(),
            options,
        )
        .await
    {
        Ok(report) => {
            for error in &report.errors {
                tracing::warn!(
                    source_id = %source.source_id(),
                    error = %error,
                    "Ingestion warning"
                );
            }

            tracing::debug!(
                source_id = %source.source_id(),
                files_found = report.files_found,
                files_processed = report.files_processed,
                orphans_deleted = report.orphans_deleted,
                errors = report.errors.len(),
                "Source ingested"
            );

            (
                report.files_processed as u64,
                report.errors.len() as u64,
                report.orphans_deleted as u64,
            )
        },
        Err(e) => {
            tracing::error!(
                source_id = %source.source_id(),
                error = %e,
                "Source ingestion failed"
            );
            (0, 1, 0)
        },
    }
}

systemprompt::traits::submit_job!(&ContentIngestionJob);
