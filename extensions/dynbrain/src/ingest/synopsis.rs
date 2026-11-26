//! Writing the synopses chunks wait for.
//!
//! Only chunks the strategy leaves without context are asked for one (under
//! `auto`, the chunks with no heading), 20 a run and eight requests at a
//! time. A transient failure is retried twice after one and two seconds; one
//! the model refuses marks the chunk, which is then embedded with the context
//! it has; a provider that is down leaves it queued, and embedding waits for
//! it; a missing chat provider lets everything embed without synopsis.

use futures_util::{StreamExt, stream};

use super::context::{ContextStrategy, clean_synopsis, synopsis_prompt};
use super::index::Indexer;
use super::model::{SynopsisTask, SynopsisWrite};
use super::provider::{Failure, Summarizer};
use super::run::RunReport;
use crate::assets::markdown::index_text;
use crate::error::BrainError;

impl Indexer {
    pub(super) async fn write_synopses(&self, report: &mut RunReport) -> Result<bool, BrainError> {
        let Some(summarizer) = self.summarizer.as_deref() else {
            return Ok(false);
        };
        let strategy = self.settings.strategy;
        if !strategy.wants_every_synopsis() && strategy != ContextStrategy::Auto {
            return Ok(true);
        }
        let tasks = self
            .repo
            .synopsis_tasks(
                self.settings.synopsis_batch,
                strategy.wants_every_synopsis(),
            )
            .await?;
        let answers: Vec<(SynopsisTask, Result<String, Failure>)> = stream::iter(tasks)
            .map(|task| async move {
                let answer = self.summarize(summarizer, &task).await;
                (task, answer)
            })
            .buffered(self.settings.synopsis_concurrency)
            .collect()
            .await;
        let mut writes = Vec::new();
        let mut refused = Vec::new();
        let mut chat_available = true;
        for (task, answer) in answers {
            match answer {
                Ok(text) => writes.push(SynopsisWrite {
                    id: task.id,
                    text,
                    content_hash: task.content_hash,
                }),
                Err(Failure::Rejected(reason)) => {
                    tracing::warn!(chunk = task.id, %reason, "dynbrain: a synopsis was refused");
                    refused.push(task.id);
                },
                Err(Failure::NotConfigured(reason)) => {
                    tracing::warn!(%reason, "dynbrain: no chat provider; chunks are embedded without synopsis");
                    chat_available = false;
                },
                Err(failure) => {
                    tracing::warn!(%failure, "dynbrain: a synopsis is waiting for the provider");
                    report.synopses_waiting += 1;
                },
            }
        }
        if !writes.is_empty() {
            report.synopses += self.repo.store_synopses(&writes).await? as usize;
        }
        if !refused.is_empty() {
            self.repo.mark_synopsis_errors(&refused).await?;
            report.synopsis_errors += refused.len();
        }
        Ok(chat_available)
    }

    async fn summarize(
        &self,
        summarizer: &dyn Summarizer,
        task: &SynopsisTask,
    ) -> Result<String, Failure> {
        let prompt = synopsis_prompt(&task.lang, &index_text(&task.body), &task.content);
        let mut backoff = self.settings.synopsis_backoff;
        let mut attempt = 0;
        loop {
            match summarizer.summarize(&prompt).await {
                Ok(text) => {
                    let clean = clean_synopsis(&text);
                    if clean.is_empty() {
                        return Err(Failure::Rejected("the model answered nothing".to_owned()));
                    }
                    return Ok(clean);
                },
                Err(failure)
                    if failure.is_transient() && attempt < self.settings.synopsis_retries =>
                {
                    attempt += 1;
                    tokio::time::sleep(backoff).await;
                    backoff *= 2;
                },
                Err(failure) => return Err(failure),
            }
        }
    }
}
