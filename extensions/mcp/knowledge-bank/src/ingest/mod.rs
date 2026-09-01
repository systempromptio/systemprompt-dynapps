//! Ingestion: read sources as the caller, chunk, embed, write the shared
//! index, and grant what was read to the caller.
//!
//! Runs inside this process when a user calls `sync_knowledge` with their own
//! Atlassian credentials — never on the serving path: a search reads the
//! index the last runs left. A run is single-flight *per caller*; two users
//! may sync at once, and a page both can read is embedded once and granted
//! twice. Sources are processed in turn, documents within a source
//! concurrently; a failing document or source is counted and the run
//! continues (NFR-DEGRADE-1). The version stamp is written last, so a reader
//! that sees a new version sees a complete index.

pub mod chunker;
pub mod dating;
pub mod document;
pub mod enrich;
pub mod html;
pub mod pipeline;
pub mod report;
pub mod sources;
pub mod writer;

mod guard;

pub use report::{IngestReport, RunOptions, SourceReport};

use crate::config::{KnowledgeBankConfig, SourceConfig};
use crate::downstream::AtlassianCredentials;
use crate::error::KnowledgeBankError;
use crate::model::ModelProvider;
use crate::principal::Principal;
use chrono::{DateTime, Utc};
use futures::StreamExt;
use guard::RunGuard;
use pipeline::{DocumentJob, process_document};
use sources::{AtlassianConnector, FetchRequest, SourceConnector};
use sqlx::PgPool;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
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

    pub async fn run(
        &self,
        caller: &Principal,
        credentials: AtlassianCredentials,
        options: &RunOptions,
    ) -> Result<IngestReport, KnowledgeBankError> {
        let connector = AtlassianConnector::new(credentials)?;
        self.run_with(caller, &connector, options).await
    }

    // Why: the connector is a parameter so the write path can be proven
    // without Atlassian; `run` is the production wrapper around it.
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
            let run = RunContext {
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

    async fn run_source(
        &self,
        id: &str,
        source: &SourceConfig,
        run: &RunContext<'_>,
    ) -> SourceReport {
        let started_at: DateTime<Utc> = Utc::now();
        let mut report = SourceReport {
            source_type: id.to_owned(),
            ..SourceReport::default()
        };
        match self.sync_source(id, source, run, &mut report).await {
            Ok(()) => {},
            Err(e) => report.error = Some(e.to_string()),
        }
        let counters = SyncCounters {
            seen: i32::try_from(report.seen).unwrap_or(i32::MAX),
            written: i32::try_from(report.written).unwrap_or(i32::MAX),
            pruned: i32::try_from(report.pruned).unwrap_or(i32::MAX),
            failed: i32::try_from(report.failed).unwrap_or(i32::MAX),
        };
        let key = SyncKey {
            source_type: id,
            user_id: &run.caller.user_id,
        };
        if let Err(e) = writer::cache::write_sync_state(
            &self.pool,
            key,
            started_at,
            counters,
            report.error.as_deref(),
        )
        .await
        {
            tracing::warn!(source = id, error = %e, "could not record sync state");
        }
        report
    }

    async fn sync_source(
        &self,
        id: &str,
        source: &SourceConfig,
        run: &RunContext<'_>,
        report: &mut SourceReport,
    ) -> Result<(), KnowledgeBankError> {
        let caller = &run.caller.user_id;
        let last_sync = if run.options.rebuild {
            None
        } else {
            writer::cache::read_last_sync(&self.pool, id, caller).await?
        };
        let known_doc_ids = writer::list_doc_ids(&self.pool, id).await?;
        let outcome = run
            .connector
            .fetch(&FetchRequest {
                source_type: id,
                source,
                last_sync,
                include_comments: self.config.atlassian.include_comments,
                known_doc_ids: &known_doc_ids,
            })
            .await?;

        report.seen = outcome.documents.len() + outcome.unchanged.len();
        for doc_id in &outcome.unchanged {
            report.unchanged += 1;
            writer::grant_to_caller(&self.pool, doc_id, caller).await?;
        }

        let counters = Counters::default();
        futures::stream::iter(&outcome.documents)
            .for_each_concurrent(self.config.ingestion.doc_concurrency.max(1), |document| {
                self.index_document(source, document, run, &counters)
            })
            .await;
        report.written = counters.written.into_inner();
        report.unchanged += counters.unchanged.into_inner();
        report.failed = counters.failed.into_inner();
        report.chunks = counters.chunks.into_inner();

        if outcome.complete {
            let dropped =
                writer::drop_stale_caller_grants(&self.pool, id, caller, &outcome.current_refs)
                    .await?;
            let orphaned = writer::delete_orphan_documents(&self.pool).await?;
            report.pruned = usize::try_from(dropped).unwrap_or(usize::MAX);
            if orphaned > 0 {
                tracing::info!(
                    source = id,
                    orphaned,
                    "documents nobody holds any more deleted"
                );
            }
        }
        Ok(())
    }

    async fn index_document(
        &self,
        source: &SourceConfig,
        document: &document::SourceDocument,
        run: &RunContext<'_>,
        counters: &Counters,
    ) {
        let caller = &run.caller.user_id;
        if !run.options.rebuild && self.is_unchanged(document).await {
            match writer::grant_to_caller(&self.pool, &document.doc_id, caller).await {
                Ok(()) => counters.unchanged.fetch_add(1, Ordering::Relaxed),
                Err(e) => {
                    tracing::warn!(doc_id = %document.doc_id, error = %e, "grant failed");
                    counters.failed.fetch_add(1, Ordering::Relaxed)
                },
            };
            return;
        }
        let job = DocumentJob {
            pool: &self.pool,
            model: self.model.as_ref(),
            config: &self.config,
            source,
            document,
        };
        let stored = match process_document(&job).await {
            Ok(count) => count,
            Err(e) => {
                tracing::warn!(doc_id = %document.doc_id, error = %e, "document failed");
                counters.failed.fetch_add(1, Ordering::Relaxed);
                return;
            },
        };
        match writer::grant_to_caller(&self.pool, &document.doc_id, caller).await {
            Ok(()) => {
                counters.written.fetch_add(1, Ordering::Relaxed);
                counters.chunks.fetch_add(stored, Ordering::Relaxed);
            },
            Err(e) => {
                tracing::warn!(doc_id = %document.doc_id, error = %e, "grant failed");
                counters.failed.fetch_add(1, Ordering::Relaxed);
            },
        }
    }

    async fn is_unchanged(&self, document: &document::SourceDocument) -> bool {
        writer::find_content_hash(&self.pool, &document.doc_id)
            .await
            .ok()
            .flatten()
            .as_deref()
            == Some(document.content_hash.as_str())
    }
}

struct RunContext<'a> {
    caller: &'a Principal,
    connector: &'a dyn SourceConnector,
    options: &'a RunOptions,
}

#[derive(Default)]
struct Counters {
    written: AtomicUsize,
    unchanged: AtomicUsize,
    failed: AtomicUsize,
    chunks: AtomicUsize,
}
