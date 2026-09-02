//! Ingestion: read sources, chunk, embed, write the shared index, and grant
//! what was read.
//!
//! Runs inside this process when a user calls `sync_knowledge` — never on
//! the serving path: a search reads the index the last runs left. Files
//! categories are read from disk and need no credential; Confluence and Jira
//! are read with the caller's own Atlassian credentials and granted to that
//! caller. A run is single-flight *per caller*; two users may sync at once,
//! and a page both can read is embedded once and granted twice. Sources are
//! processed in turn, documents within a source concurrently; a failing
//! document or source is counted and the run continues (NFR-DEGRADE-1). The
//! version stamp is written last, so a reader that sees a new version sees a
//! complete index. The per-source loop lives in the private `run` module, the
//! per-document pipeline in [`pipeline`], and the immediate single-document
//! path the upload tool uses in [`upload`].

pub mod chunker;
pub mod dating;
pub mod document;
pub mod enrich;
pub mod grants;
pub mod html;
pub mod parsers;
pub mod pipeline;
pub mod report;
pub mod sources;
pub mod upload;
pub mod writer;

mod guard;
mod run;

pub use report::{IngestReport, RunOptions, SourceReport};
pub use upload::{MAX_UPLOAD_BYTES, UploadReport, UploadRequest};

use crate::config::KnowledgeBankConfig;
use crate::downstream::AtlassianCredentials;
use crate::error::KnowledgeBankError;
use crate::model::ModelProvider;
use crate::principal::Principal;
use guard::RunGuard;
use sources::{ConfiguredConnector, SourceConnector};
use sqlx::PgPool;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use systemprompt::identifiers::UserId;

/// The ingestion runner: owns the handles, guarantees single flight per user.
pub struct Ingestor {
    pool: Arc<PgPool>,
    model: Arc<dyn ModelProvider>,
    config: Arc<KnowledgeBankConfig>,
    running: Mutex<HashSet<UserId>>,
}

impl std::fmt::Debug for Ingestor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ingestor").finish_non_exhaustive()
    }
}

impl Ingestor {
    pub fn new(
        pool: Arc<PgPool>,
        model: Arc<dyn ModelProvider>,
        config: Arc<KnowledgeBankConfig>,
    ) -> Self {
        Self {
            pool,
            model,
            config,
            running: Mutex::new(HashSet::new()),
        }
    }

    #[must_use]
    pub fn is_running_for(&self, user_id: &UserId) -> bool {
        guard::lock_running(&self.running).contains(user_id)
    }

    #[must_use]
    pub fn config(&self) -> &KnowledgeBankConfig {
        &self.config
    }

    // Why: `credentials` may be absent when the run is limited to files
    // categories; an Atlassian category reached without them fails for that
    // source alone.
    pub async fn run(
        &self,
        caller: &Principal,
        credentials: Option<AtlassianCredentials>,
        options: &RunOptions,
    ) -> Result<IngestReport, KnowledgeBankError> {
        let connector = ConfiguredConnector::new(credentials)?;
        self.run_with(caller, &connector, options).await
    }

    // Why: the connector is a parameter so the write path can be proven
    // without Atlassian or a directory tree; `run` is the production wrapper.
    pub async fn run_with(
        &self,
        caller: &Principal,
        connector: &dyn SourceConnector,
        options: &RunOptions,
    ) -> Result<IngestReport, KnowledgeBankError> {
        let _guard = RunGuard::acquire(&self.running, caller.user_id.clone())?;
        let started = std::time::Instant::now();
        let mut report = IngestReport::default();

        for (id, source) in &self.config.sources {
            if !options.sources.is_empty() && !options.sources.contains(id) {
                continue;
            }
            let run = run::RunContext {
                caller,
                connector,
                options,
            };
            let source_report = self.run_source(id, source, &run).await;
            tracing::info!(
                source = id,
                user_id = %caller.user_id,
                seen = source_report.seen,
                written = source_report.written,
                unchanged = source_report.unchanged,
                failed = source_report.failed,
                pruned = source_report.pruned,
                error = source_report.error.as_deref().unwrap_or(""),
                "knowledge source synced"
            );
            report.sources.push(source_report);
        }

        report.version = Some(writer::stamp_index_state(&self.pool).await?);
        report.duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        Ok(report)
    }
}
