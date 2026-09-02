//! One source category, one run: fetch, skip the unchanged, index the rest
//! concurrently, grant, and prune what disappeared.

use super::document::SourceDocument;
use super::grants::{self, GrantPolicy};
use super::pipeline::{DocumentJob, process_document};
use super::sources::{FetchRequest, SourceConnector};
use super::writer::cache::{SyncCounters, SyncKey};
use super::{Ingestor, RunOptions, SourceReport, writer};
use crate::config::{SourceConfig, SourceKind};
use crate::error::KnowledgeBankError;
use crate::principal::Principal;
use chrono::{DateTime, Utc};
use futures::StreamExt;
use std::sync::atomic::{AtomicUsize, Ordering};

pub(super) struct RunContext<'a> {
    pub(super) caller: &'a Principal,
    pub(super) connector: &'a dyn SourceConnector,
    pub(super) options: &'a RunOptions,
}

struct SourceScope<'a> {
    source: &'a SourceConfig,
    policy: GrantPolicy<'a>,
}

#[derive(Default)]
struct Counters {
    written: AtomicUsize,
    unchanged: AtomicUsize,
    failed: AtomicUsize,
    chunks: AtomicUsize,
}

impl Ingestor {
    pub(super) async fn run_source(
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
        let policy = grants::policy_for(source, caller);
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
                files: &self.config.files,
                last_sync,
                include_comments: self.config.atlassian.include_comments,
                known_doc_ids: &known_doc_ids,
            })
            .await?;

        report.seen = outcome.documents.len() + outcome.unchanged.len();
        for doc_id in &outcome.unchanged {
            report.unchanged += 1;
            writer::apply_grants(&self.pool, doc_id, &policy).await?;
        }

        let counters = Counters::default();
        let scope = SourceScope { source, policy };
        futures::stream::iter(&outcome.documents)
            .for_each_concurrent(self.config.ingestion.doc_concurrency.max(1), |document| {
                self.index_document(&scope, document, run, &counters)
            })
            .await;
        report.written = counters.written.into_inner();
        report.unchanged += counters.unchanged.into_inner();
        report.failed = counters.failed.into_inner();
        report.chunks = counters.chunks.into_inner();

        if outcome.complete {
            report.pruned = self
                .prune_vanished(id, source, caller, &outcome.current_refs)
                .await?;
        }
        Ok(())
    }

    // Why: files documents carry configured grants, not the caller's, so the
    // per-caller grant drop can never free them; a complete listing deletes
    // the vanished files directly (FR-ING-7).
    async fn prune_vanished(
        &self,
        id: &str,
        source: &SourceConfig,
        caller: &systemprompt::identifiers::UserId,
        current_refs: &[String],
    ) -> Result<usize, KnowledgeBankError> {
        let removed = match source.kind {
            SourceKind::Files => {
                writer::delete_missing_documents(&self.pool, id, current_refs).await?
            },
            SourceKind::Confluence | SourceKind::Jira => {
                let dropped =
                    writer::drop_stale_caller_grants(&self.pool, id, caller, current_refs).await?;
                let orphaned = writer::delete_orphan_documents(&self.pool).await?;
                if orphaned > 0 {
                    tracing::info!(
                        source = id,
                        orphaned,
                        "documents nobody holds any more deleted"
                    );
                }
                dropped
            },
        };
        Ok(usize::try_from(removed).unwrap_or(usize::MAX))
    }

    async fn index_document(
        &self,
        scope: &SourceScope<'_>,
        document: &SourceDocument,
        run: &RunContext<'_>,
        counters: &Counters,
    ) {
        let SourceScope { source, policy } = scope;
        if !run.options.rebuild && self.is_unchanged(document).await {
            match writer::apply_grants(&self.pool, &document.doc_id, policy).await {
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
        let granted = match (run.options.rebuild, policy) {
            (true, GrantPolicy::Configured(subjects)) => {
                writer::replace_configured_grants(&self.pool, &document.doc_id, subjects).await
            },
            _ => writer::apply_grants(&self.pool, &document.doc_id, policy).await,
        };
        match granted {
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

    async fn is_unchanged(&self, document: &SourceDocument) -> bool {
        writer::find_content_hash(&self.pool, &document.doc_id)
            .await
            .ok()
            .flatten()
            .as_deref()
            == Some(document.content_hash.as_str())
    }
}
