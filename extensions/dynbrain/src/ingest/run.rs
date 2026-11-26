//! The knobs of an indexing run and what a run reports.

use std::time::Duration;

use super::context::ContextStrategy;
use super::provider::Failure;

/// The numbers of dynbrain's indexing, which are the specification: jobs in
/// batches of 200 (five rounds a run), synopses 20 at a time with 8 in
/// flight, retried twice after 1 s and 2 s, and embeddings in batches of 50.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexSettings {
    pub strategy: ContextStrategy,
    pub job_batch: i64,
    pub synopsis_batch: i64,
    pub synopsis_concurrency: usize,
    pub synopsis_retries: u32,
    pub synopsis_backoff: Duration,
    pub embed_batch: usize,
    pub embed_rounds: usize,
    pub max_attempts: i32,
}

impl Default for IndexSettings {
    fn default() -> Self {
        Self {
            strategy: ContextStrategy::Auto,
            job_batch: 200,
            synopsis_batch: 20,
            synopsis_concurrency: 8,
            synopsis_retries: 2,
            synopsis_backoff: Duration::from_secs(1),
            embed_batch: 50,
            embed_rounds: 20,
            max_attempts: 5,
        }
    }
}

/// What one run did.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RunReport {
    pub queued: u64,
    pub indexed: usize,
    pub purged: usize,
    pub failed: usize,
    pub synopses: usize,
    pub synopsis_errors: usize,
    pub synopses_waiting: usize,
    pub embedded: usize,
    pub rejected: usize,
    pub cost_microdollars: i64,
    pub other_fingerprints: i64,
    pub stopped: Option<Failure>,
}

impl RunReport {
    pub fn summary(&self) -> String {
        let mut parts = vec![format!(
            "{} variant(s) indexed, {} left out of the index, {} failed; {} synopsis(es), {} chunk(s) embedded for {} microdollars, {} refused",
            self.indexed,
            self.purged,
            self.failed,
            self.synopses,
            self.embedded,
            self.cost_microdollars,
            self.rejected
        )];
        if self.synopses_waiting > 0 {
            parts.push(format!(
                "{} synopsis(es) waiting for the provider",
                self.synopses_waiting
            ));
        }
        if self.other_fingerprints > 0 {
            parts.push(format!(
                "{} chunk(s) were embedded with another model and are not searched by vector",
                self.other_fingerprints
            ));
        }
        if let Some(stopped) = &self.stopped {
            parts.push(format!(
                "embedding stopped, the rest stays queued: {stopped}"
            ));
        }
        parts.join("; ")
    }
}
