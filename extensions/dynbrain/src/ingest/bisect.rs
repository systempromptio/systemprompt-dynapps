//! Embedding a batch, and finding the one text a provider refuses.
//!
//! A batch goes as one request. When it is refused, it is split in half and
//! each half is tried once; only a half whose sibling embedded something is
//! split further, because that sibling's success proves the provider accepts
//! requests and the failing half really is about its own text. If neither
//! half embeds anything, the failure says nothing about any text (a rejected
//! key, a model that no longer exists, vectors of the wrong size): the run
//! stops there, after three requests, and marks nobody. Isolating one bad
//! text out of `n` costs about `log n` extra requests, not `n`.
//!
//! A failure that is about the provider (a rate limit, an outage, a missing
//! key) stops the run at once and marks nobody either: what was embedded so
//! far is kept, and the rest stays queued for the next run.

use super::provider::{DIMENSIONS, Embedder, Embedding, Failure};

/// One text to embed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub id: i64,
    pub text: String,
    pub context_hash: String,
}

/// A vector for one candidate.
#[derive(Debug, Clone, PartialEq)]
pub struct Vectored {
    pub id: i64,
    pub vector: Vec<f32>,
    pub context_hash: String,
}

/// What one successful request cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spend {
    pub inputs: usize,
    pub input_tokens: u32,
    pub cost_microdollars: i64,
}

/// What a batch came to: vectors, the ids the provider refused, the requests
/// that were billed, and why the run stopped, if it did.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct EmbeddingRun {
    pub vectors: Vec<Vectored>,
    pub rejected: Vec<i64>,
    pub spends: Vec<Spend>,
    pub stopped: Option<Failure>,
}

pub async fn embed_bisecting(embedder: &dyn Embedder, items: &[Candidate]) -> EmbeddingRun {
    let mut run = EmbeddingRun::default();
    let mut pending = vec![(0, items.len(), false)];
    while let Some((from, to, known_failing)) = pending.pop() {
        let slice = &items[from..to];
        if slice.is_empty() {
            continue;
        }
        let embedded = if known_failing {
            0
        } else {
            attempt(embedder, slice, &mut run).await
        };
        if run.stopped.is_some() {
            break;
        }
        if embedded > 0 || slice.len() == 1 {
            if embedded == 0 {
                run.rejected.push(slice[0].id);
            }
            continue;
        }
        let middle = from + slice.len() / 2;
        let left = attempt(embedder, &items[from..middle], &mut run).await;
        if run.stopped.is_some() {
            break;
        }
        let right = attempt(embedder, &items[middle..to], &mut run).await;
        if run.stopped.is_some() {
            break;
        }
        if right == 0 && left > 0 {
            pending.push((middle, to, true));
        }
        if left == 0 && right > 0 {
            pending.push((from, middle, true));
        }
    }
    run
}

async fn attempt(embedder: &dyn Embedder, slice: &[Candidate], run: &mut EmbeddingRun) -> usize {
    let texts: Vec<String> = slice.iter().map(|item| item.text.clone()).collect();
    match embedder.embed(&texts).await {
        Ok(done) if usable(&done, slice.len()) => {
            let accepted = done.vectors.len();
            run.spends.push(Spend {
                inputs: texts.len(),
                input_tokens: done.input_tokens,
                cost_microdollars: done.cost_microdollars,
            });
            run.vectors.extend(
                slice
                    .iter()
                    .zip(done.vectors)
                    .map(|(item, vector)| Vectored {
                        id: item.id,
                        vector,
                        context_hash: item.context_hash.clone(),
                    }),
            );
            accepted
        },
        Ok(done) => {
            tracing::warn!(
                asked = slice.len(),
                returned = done.vectors.len(),
                "dynbrain: the embedding provider returned vectors the index cannot hold"
            );
            0
        },
        Err(failure) if failure.is_systemic() => {
            tracing::warn!(%failure, "dynbrain: embedding stopped, the chunks stay queued");
            run.stopped = Some(failure);
            0
        },
        Err(failure) => {
            tracing::warn!(%failure, asked = slice.len(), "dynbrain: a batch was refused");
            0
        },
    }
}

fn usable(done: &Embedding, asked: usize) -> bool {
    !done.vectors.is_empty()
        && done.vectors.len() <= asked
        && done.vectors.iter().all(|vector| vector.len() == DIMENSIONS)
}
