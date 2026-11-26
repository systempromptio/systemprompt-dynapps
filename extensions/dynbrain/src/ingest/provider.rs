//! What the indexer asks of the models, behind two traits so the job runs
//! against deterministic doubles in tests and against core's `AiService` in
//! production ([`super::core_ai`]).
//!
//! A [`Failure`] says what to do about a failed call: `RateLimited`,
//! `Unavailable` and `NotConfigured` are about the provider, not the text,
//! so the run stops and the chunks stay queued; `Rejected` is about the text
//! that was sent.

use async_trait::async_trait;

use super::context::SynopsisPrompt;

// Why: the index column is `vector(1536)`; a vector of another size cannot be
// stored.
pub const DIMENSIONS: usize = 1536;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Failure {
    #[error("the provider asked to slow down")]
    RateLimited,
    #[error("the provider is unavailable: {0}")]
    Unavailable(String),
    #[error("no provider is configured for this model: {0}")]
    NotConfigured(String),
    #[error("the provider rejected the request: {0}")]
    Rejected(String),
}

impl Failure {
    pub const fn is_transient(&self) -> bool {
        matches!(self, Self::RateLimited | Self::Unavailable(_))
    }

    pub const fn is_systemic(&self) -> bool {
        !matches!(self, Self::Rejected(_))
    }
}

/// The vectors of one call, in input order, with what the call cost.
#[derive(Debug, Clone, PartialEq)]
pub struct Embedding {
    pub vectors: Vec<Vec<f32>>,
    pub input_tokens: u32,
    pub cost_microdollars: i64,
}

/// Turns texts into vectors. Held as `Arc<dyn Embedder>` so tests can swap it,
/// hence `#[async_trait]`.
#[async_trait]
pub trait Embedder: Send + Sync {
    fn fingerprint(&self) -> String;

    async fn embed(&self, texts: &[String]) -> Result<Embedding, Failure>;
}

/// Writes the synopsis of a chunk. Held as `Arc<dyn Summarizer>` so tests can
/// swap it, hence `#[async_trait]`.
#[async_trait]
pub trait Summarizer: Send + Sync {
    async fn summarize(&self, prompt: &SynopsisPrompt) -> Result<String, Failure>;
}
