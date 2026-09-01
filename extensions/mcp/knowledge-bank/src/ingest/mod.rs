//! Ingestion: read sources, chunk, embed, write the index.
//!
//! Ingestion: read every configured source, chunk, embed, and write the
//! index. Runs inside this process — on the configured schedule, from the
//! `ingest` subcommand, or from the admin-only `sync_knowledge` tool — and
//! never on the serving path: a tool call reads the index the last run left.
//!
//! One run is single-flight. Sources are processed in turn, documents within
//! a source concurrently; a failing document or source is counted and the
//! run continues, because indexing must never hard-fail on a single bad
//! input (NFR-DEGRADE-1). The version stamp is written last, so a reader that
//! sees a new version sees a complete index.

pub mod chunker;
pub mod dating;
pub mod document;
pub mod enrich;
pub mod grants;
pub mod html;
pub mod parsers;
pub mod pipeline;
pub mod scheduler;
pub mod sources;
pub mod writer;

use crate::config::{KnowledgeBankConfig, SourceConfig};
use crate::error::KnowledgeBankError;
use crate::model::ModelProvider;
use chrono::{DateTime, Utc};
use futures::StreamExt;
use grants::{GrantResolver, IdentityMap};
use pipeline::{DocumentJob, process_document};
use sources::{AtlassianClient, FetchRequest};
use sqlx::PgPool;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Mutex;
use writer::cache::SyncCounters;

/// What one run should do.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunOptions {
    pub rebuild: bool,
    pub sources: Vec<String>,
}

/// What one run did to one source.
#[derive(Debug, Clone, Default)]
pub struct SourceReport {
    pub source_type: String,
    pub seen: usize,
    pub written: usize,
    pub unchanged: usize,
    pub failed: usize,
    pub pruned: usize,
    pub chunks: usize,
    pub error: Option<String>,
}

/// What one run did overall.
#[derive(Debug, Clone, Default)]
pub struct IngestReport {
    pub sources: Vec<SourceReport>,
    pub version: Option<String>,
    pub duration_ms: u64,
}

impl IngestReport {
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.sources.iter().any(|s| s.error.is_some())
    }
}

/// The ingestion runner: owns the handles, guarantees single flight.
pub struct Ingestor {
    pool: Arc<PgPool>,
    model: Arc<dyn ModelProvider>,
    config: Arc<KnowledgeBankConfig>,
    atlassian: Option<AtlassianClient>,
    running: Mutex<()>,
}

impl std::fmt::Debug for Ingestor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ingestor")
            .field("atlassian", &self.atlassian.is_some())
            .finish_non_exhaustive()
    }
}

impl Ingestor {
    pub fn new(
        pool: Arc<PgPool>,
        model: Arc<dyn ModelProvider>,
        config: Arc<KnowledgeBankConfig>,
        atlassian: Option<AtlassianClient>,
    ) -> Self {
        Self {
            pool,
            model,
            config,
            atlassian,
            running: Mutex::new(()),
        }
    }

    #[must_use]
    pub fn is_running(&self) -> bool {
        self.running.try_lock().is_err()
    }

    #[must_use]
    pub fn config(&self) -> &KnowledgeBankConfig {
        &self.config
    }

    pub async fn run(&self, options: &RunOptions) -> Result<IngestReport, KnowledgeBankError> {
        let Ok(_guard) = self.running.try_lock() else {
            return Err(KnowledgeBankError::Backend(
                "an ingestion run is already in progress".to_owned(),
            ));
        };
        let started = std::time::Instant::now();
        let identities = IdentityMap::load(&self.pool).await?;
        let mut report = IngestReport::default();

        for (id, source) in &self.config.sources {
            if !options.sources.is_empty() && !options.sources.contains(id) {
                continue;
            }
            let run = RunContext {
                options,
                identities: &identities,
            };
            let source_report = self.run_source(id, source, &run).await;
            tracing::info!(
                source = id,
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
        if let Err(e) = writer::cache::write_sync_state(
            &self.pool,
            id,
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
        let (options, identities) = (run.options, run.identities);
        if options.rebuild {
            writer::delete_source(&self.pool, id).await?;
            writer::cache::clear_sync_cursor(&self.pool, id).await?;
        }
        let last_sync = if options.rebuild {
            None
        } else {
            writer::cache::read_last_sync(&self.pool, id).await?
        };
        let resolver = GrantResolver::new(source, &self.config.atlassian, identities.clone());
        let outcome = sources::fetch(&FetchRequest {
            source_type: id,
            source,
            grants: &resolver,
            last_sync,
            include_comments: self.config.atlassian.include_comments,
            atlassian: self.atlassian.as_ref(),
        })
        .await?;

        report.seen = outcome.documents.len() + outcome.unchanged.len();
        for unchanged in &outcome.unchanged {
            report.unchanged += 1;
            if let Some(grants) = &unchanged.grants {
                writer::rewrite_grants(&self.pool, &unchanged.doc_id, grants).await?;
            }
        }

        let counters = Counters::default();
        futures::stream::iter(&outcome.documents)
            .for_each_concurrent(self.config.ingestion.doc_concurrency.max(1), |document| {
                self.index_document(source, document, options.rebuild, &counters)
            })
            .await;
        report.written = counters.written.into_inner();
        report.unchanged += counters.unchanged.into_inner();
        report.failed = counters.failed.into_inner();
        report.chunks = counters.chunks.into_inner();

        if outcome.complete {
            let pruned = writer::prune_missing(&self.pool, id, &outcome.current_refs).await?;
            report.pruned = usize::try_from(pruned).unwrap_or(usize::MAX);
        }
        Ok(())
    }

    async fn index_document(
        &self,
        source: &SourceConfig,
        document: &document::SourceDocument,
        rebuild: bool,
        counters: &Counters,
    ) {
        if !rebuild && self.is_unchanged(document).await {
            if let Err(e) =
                writer::rewrite_grants(&self.pool, &document.doc_id, &document.grants).await
            {
                tracing::warn!(doc_id = %document.doc_id, error = %e, "grant refresh failed");
            }
            counters.unchanged.fetch_add(1, Ordering::Relaxed);
            return;
        }
        if document.grants.is_empty() {
            tracing::warn!(
                doc_id = %document.doc_id,
                "indexing with no grants: restrictions could not be translated, so this \
                 document will be reachable by nobody"
            );
        }
        let job = DocumentJob {
            pool: &self.pool,
            model: self.model.as_ref(),
            config: &self.config,
            source,
            document,
        };
        match process_document(&job).await {
            Ok(count) => {
                counters.written.fetch_add(1, Ordering::Relaxed);
                counters.chunks.fetch_add(count, Ordering::Relaxed);
            },
            Err(e) => {
                tracing::warn!(doc_id = %document.doc_id, error = %e, "document failed");
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
    options: &'a RunOptions,
    identities: &'a IdentityMap,
}

#[derive(Default)]
struct Counters {
    written: AtomicUsize,
    unchanged: AtomicUsize,
    failed: AtomicUsize,
    chunks: AtomicUsize,
}
