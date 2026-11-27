//! The indexer: one run works the queue, writes the synopses chunks are
//! waiting for and embeds what has no vector yet.
//!
//! A run is safe to repeat and to interrupt: a job is settled only after its
//! chunks are stored, a synopsis only after the model answered, a vector only
//! after the provider returned it, and everything unfinished stays queued.
//! It never calls a model inside a request: only the scheduler's job does.

use std::fmt;
use std::sync::Arc;

use super::bisect::{Candidate, embed_bisecting};
use super::chunker::chunk_document;
use super::context::{
    ChunkContext, ContextStrategy, document_context, embedding_text, prefix, text_hash,
};
use super::eligibility::{Placement, eligible};
use super::model::{EmbedTask, NewChunk, QueuedJob};
use super::provider::{Embedder, Summarizer};
use super::run::{IndexSettings, RunReport};
use crate::assets::markdown::index_text;
use crate::error::BrainError;
use crate::language::fts_config;
use crate::repository::BrainRepository;

const ENQUEUE_PER_RUN: i64 = 500;
const JOB_ROUNDS: usize = 5;

enum Outcome {
    Indexed,
    Purged,
    Skipped,
}

pub struct Indexer {
    pub(super) repo: BrainRepository,
    pub(super) embedder: Arc<dyn Embedder>,
    pub(super) summarizer: Option<Arc<dyn Summarizer>>,
    pub(super) settings: IndexSettings,
}

impl fmt::Debug for Indexer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Indexer")
            .field("settings", &self.settings)
            .finish_non_exhaustive()
    }
}

impl Indexer {
    pub fn new(
        repo: BrainRepository,
        embedder: Arc<dyn Embedder>,
        summarizer: Option<Arc<dyn Summarizer>>,
        settings: IndexSettings,
    ) -> Self {
        Self {
            repo,
            embedder,
            summarizer,
            settings,
        }
    }

    pub async fn run(&self) -> Result<RunReport, BrainError> {
        let mut report = RunReport::default();
        self.repo.reset_running().await?;
        report.queued = self.repo.enqueue_missing(ENQUEUE_PER_RUN).await?;
        self.index_jobs(&mut report).await?;
        let chat_available = self.write_synopses(&mut report).await?;
        self.embed(&mut report, chat_available).await?;
        self.count_other_fingerprints(&mut report).await?;
        Ok(report)
    }

    async fn index_jobs(&self, report: &mut RunReport) -> Result<(), BrainError> {
        for _ in 0..JOB_ROUNDS {
            let jobs = self.repo.claim_index_jobs(self.settings.job_batch).await?;
            if jobs.is_empty() {
                break;
            }
            for job in &jobs {
                match self.index_one(job).await {
                    Ok(Outcome::Indexed) => report.indexed += 1,
                    Ok(Outcome::Purged) => report.purged += 1,
                    Ok(Outcome::Skipped) => {},
                    Err(error) => {
                        tracing::warn!(?error, article = %job.article, lang = %job.lang, "dynbrain: indexing failed");
                        let detail = format!("{error:?}");
                        let attempts = self.settings.max_attempts;
                        self.repo.fail_job(job.id, &detail, attempts).await?;
                        report.failed += 1;
                    },
                }
            }
        }
        Ok(())
    }

    async fn index_one(&self, job: &QueuedJob) -> Result<Outcome, BrainError> {
        let Some(source) = self.repo.index_source(job.article, &job.lang).await? else {
            self.repo.drop_job(job.id).await?;
            return Ok(Outcome::Skipped);
        };
        if source.trashed {
            self.repo.release_job(job.id).await?;
            return Ok(Outcome::Skipped);
        }
        let placement = Placement {
            space: source.space,
            lineage: self.repo.lineage(job.article).await?,
        };
        let rules = self.repo.list_rag_rules().await?;
        if !eligible(source.rag_mode, &rules, &placement) {
            self.repo.purge_chunks(job.article, &job.lang).await?;
            self.repo.settle_job(job, &source.content_hash).await?;
            return Ok(Outcome::Purged);
        }
        let current = self
            .repo
            .is_indexed(job.article, &job.lang, &source.content_hash)
            .await?;
        if !current {
            let carried = self.repo.carried_synopses(job.article, &job.lang).await?;
            let chunks: Vec<NewChunk> = chunk_document(&index_text(&source.body))
                .into_iter()
                .map(|piece| {
                    let content_hash = text_hash(&piece.content);
                    NewChunk {
                        synopsis: carried.get(&content_hash).cloned(),
                        heading_path: piece.heading_path,
                        content: piece.content,
                        content_hash,
                    }
                })
                .collect();
            let lang = fts_config(&job.lang);
            self.repo
                .replace_chunks(job.article, &job.lang, lang, &chunks)
                .await?;
        }
        self.repo.settle_job(job, &source.content_hash).await?;
        Ok(if current {
            Outcome::Skipped
        } else {
            Outcome::Indexed
        })
    }

    async fn embed(&self, report: &mut RunReport, chat_available: bool) -> Result<(), BrainError> {
        let strategy = self.settings.strategy;
        let wait_for_all = chat_available && strategy.wants_every_synopsis();
        let wait_for_headingless = chat_available && strategy == ContextStrategy::Auto;
        let fingerprint = self.embedder.fingerprint();
        let batch = self.settings.embed_batch as i64;
        for _ in 0..self.settings.embed_rounds {
            let tasks = self
                .repo
                .embed_tasks(batch, wait_for_all, wait_for_headingless)
                .await?;
            if tasks.is_empty() {
                break;
            }
            let candidates: Vec<Candidate> =
                tasks.iter().map(|task| self.candidate(task)).collect();
            let run = embed_bisecting(self.embedder.as_ref(), &candidates).await;
            if !run.vectors.is_empty() {
                self.repo
                    .store_embeddings(&run.vectors, &fingerprint)
                    .await?;
            }
            if !run.rejected.is_empty() {
                self.repo.mark_embedding_errors(&run.rejected).await?;
            }
            for spend in &run.spends {
                self.repo
                    .record_embedding_spend(&fingerprint, spend)
                    .await?;
                report.cost_microdollars += spend.cost_microdollars;
            }
            report.embedded += run.vectors.len();
            report.rejected += run.rejected.len();
            let progressed = !run.vectors.is_empty() || !run.rejected.is_empty();
            if let Some(stopped) = run.stopped {
                report.stopped = Some(stopped);
                break;
            }
            if !progressed {
                break;
            }
        }
        Ok(())
    }

    fn candidate(&self, task: &EmbedTask) -> Candidate {
        let document = document_context(&task.title, &index_text(&task.description));
        let chunk = ChunkContext {
            content: &task.content,
            heading_path: &task.heading_path,
            synopsis: task.synopsis.as_deref(),
        };
        let lead = prefix(self.settings.strategy, &document, &chunk);
        Candidate {
            id: task.id,
            text: embedding_text(&lead, &task.content),
            context_hash: text_hash(&lead),
        }
    }

    async fn count_other_fingerprints(&self, report: &mut RunReport) -> Result<(), BrainError> {
        let ours = self.embedder.fingerprint();
        report.other_fingerprints = self
            .repo
            .fingerprint_counts()
            .await?
            .into_iter()
            .filter(|count| count.fingerprint != ours)
            .map(|count| count.chunks)
            .sum();
        if report.other_fingerprints > 0 {
            tracing::warn!(
                chunks = report.other_fingerprints,
                current = %ours,
                "dynbrain: chunks embedded with another configuration are left out of the vector search"
            );
        }
        Ok(())
    }
}
