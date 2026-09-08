//! Ingestion: read sources, chunk, embed, write the shared index, and grant
//! what was read.
//!
//! Runs inside this process when a user calls `sync_knowledge` — never on
//! the serving path: a search reads the index the last runs left. Files
//! categories are read from disk and need nothing from the caller; Confluence
//! and Jira are read through the caller's connected Atlassian MCP and granted
//! to that caller. A run is single-flight *per caller*; two users may sync at
//! once, and a page both can read is embedded once and granted twice. Sources
//! are processed in turn, documents within a source concurrently; a failing
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
use crate::error::KnowledgeBankError;
use crate::model::ModelProvider;
use crate::principal::Principal;
use guard::RunGuard;
use sources::{AtlassianMcp, ConfiguredConnector, SourceConnector};
use sqlx::PgPool;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use systemprompt::identifiers::UserId;
use writer::cache::{SyncCounters, SyncKey};

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

    // Why: `atlassian` may be absent when the run is limited to files
    // categories; an Atlassian category reached without it fails for that
    // source alone.
    pub async fn run(
        &self,
        caller: &Principal,
        atlassian: Option<AtlassianMcp>,
        options: &RunOptions,
    ) -> Result<IngestReport, KnowledgeBankError> {
        let connector = ConfiguredConnector::new(atlassian);
        self.run_with(caller, &connector, options).await
    }

    // Why: a category left out of a run because Atlassian was unreachable is
    // still reported to the caller by `list_knowledge_sources`, with the
    // reason, rather than looking never-attempted.
    pub async fn record_skipped(&self, caller: &UserId, source_ids: &[String], reason: &str) {
        for id in source_ids {
            let key = SyncKey {
                source_type: id,
                user_id: caller,
            };
            if let Err(e) = writer::cache::write_sync_state(
                &self.pool,
                key,
                chrono::Utc::now(),
                SyncCounters::default(),
                Some(reason),
            )
            .await
            {
                tracing::warn!(source = id, error = %e, "could not record the skipped source");
            }
        }
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
