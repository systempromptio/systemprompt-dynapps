//! The `dynbrain_upload_purge` job: uploads nobody adopted.
//!
//! An upload slot takes one POST of a file before its link expires, and then
//! one adoption as a file of an article. A received file that is never
//! adopted would keep its bytes in core's file storage for good, so the job
//! deletes the slots whose link expired more than [`GRACE`] (a day) ago and
//! that no article adopted, and their bytes. A file that was adopted is not
//! touched: it belongs to its article. A slot that was never filled has no
//! bytes.

use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use systemprompt::database::DbPool;
use systemprompt::extension::ExtensionContext;
use systemprompt::traits::{
    FileStorage, FileStorageError, Job, JobContext, JobResult, MissingDependency, ProviderError,
    StoredFileId,
};

use crate::error::BrainError;
use crate::repository::BrainRepository;

pub const JOB_NAME: &str = "dynbrain_upload_purge";

// Why: a person may adopt a file received just before its link expired.
pub const GRACE: Duration = Duration::hours(24);

/// What a purge removed: slots, and the files of those slots that had bytes
/// (deleted, or that storage refused to delete).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PurgeReport {
    pub slots: usize,
    pub deleted: usize,
    pub kept: usize,
}

pub async fn purge_uploads(
    repo: &BrainRepository,
    storage: &dyn FileStorage,
    before: DateTime<Utc>,
) -> Result<PurgeReport, BrainError> {
    let stored = repo.purge_expired_uploads(before).await?;
    let mut report = PurgeReport {
        slots: stored.len(),
        ..PurgeReport::default()
    };
    for key in stored.iter().flatten() {
        match storage.delete(&StoredFileId::new(key.as_str())).await {
            Ok(()) | Err(FileStorageError::NotFound(_)) => report.deleted += 1,
            Err(error) => {
                tracing::warn!(%error, "dynbrain: the bytes of an expired upload were not deleted");
                report.kept += 1;
            },
        }
    }
    Ok(report)
}

#[derive(Debug, thiserror::Error)]
pub enum PurgeJobError {
    #[error(transparent)]
    Dependency(#[from] MissingDependency),
    #[error("the Brain store failed")]
    Store(#[from] BrainError),
}

impl From<PurgeJobError> for ProviderError {
    fn from(error: PurgeJobError) -> Self {
        Self::Internal(Box::new(error))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DynbrainUploadPurgeJob;

impl DynbrainUploadPurgeJob {
    async fn run(ctx: &JobContext) -> Result<JobResult, PurgeJobError> {
        let repo = BrainRepository::new(ctx.get::<DbPool>()?.write_pool());
        let storage = ctx.get::<Arc<dyn ExtensionContext>>()?.file_storage();
        let report = purge_uploads(&repo, storage.as_ref(), Utc::now() - GRACE).await?;
        Ok(JobResult::success()
            .with_stats(report.deleted as u64, report.kept as u64)
            .with_message(format!(
                "{} expired upload(s) removed, {} file(s) deleted, {} kept",
                report.slots, report.deleted, report.kept
            )))
    }
}

#[async_trait::async_trait]
impl Job for DynbrainUploadPurgeJob {
    fn name(&self) -> &'static str {
        JOB_NAME
    }

    fn description(&self) -> &'static str {
        "Deletes the uploads of the native Brain whose link expired and that no article adopted"
    }

    fn schedule(&self) -> &'static str {
        "0 15 4 * * *"
    }

    async fn execute(&self, ctx: &JobContext) -> Result<JobResult, ProviderError> {
        tracing::info!(actor = %ctx.actor().user_id, "Brain upload purge invoked");
        Ok(Self::run(ctx).await?)
    }
}

systemprompt::traits::submit_job!(&DynbrainUploadPurgeJob);
