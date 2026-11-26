//! The query vector: embedded once per distinct query, then cached.
//!
//! The cache key is `sha1(provider:model:normalised query)`, the query
//! lower-cased with its whitespace collapsed. A hit refreshes `last_used`; a
//! miss embeds the query with a 6 s limit and one retry, stores the vector
//! and purges the entries nobody used in 30 days. Only the vector is cached,
//! never a result.
//!
//! [`EmbedderSlot`] holds the embedder. Core hands its `AiService` to the
//! extension's routes with each request, so the first search fills the slot;
//! tests fill it up front with a double.

use std::fmt;
use std::sync::{Arc, OnceLock};

use sha1::{Digest, Sha1};

use crate::ingest::provider::Embedder;

/// A query embedded, as Postgres reads a vector: `[0.1,0.2,...]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryVector(pub String);

/// The cache entry of a query: its hash, its provider and its model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryKey {
    pub hash: String,
    pub provider: String,
    pub model: String,
}

pub fn cache_key(fingerprint: &str, query: &str) -> QueryKey {
    let normalized = query
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let configuration = fingerprint
        .rsplit_once(':')
        .map_or(fingerprint, |(head, _)| head);
    let (provider, model) = configuration.split_once(':').unwrap_or((configuration, ""));
    let digest = Sha1::digest(format!("{configuration}:{normalized}").as_bytes());
    QueryKey {
        hash: hex::encode(digest),
        provider: provider.to_owned(),
        model: model.to_owned(),
    }
}

pub fn literal(vector: &[f32]) -> QueryVector {
    let parts: Vec<String> = vector.iter().map(ToString::to_string).collect();
    QueryVector(format!("[{}]", parts.join(",")))
}

/// The query embedder, filled once per process.
#[derive(Clone, Default)]
pub struct EmbedderSlot(Arc<OnceLock<Arc<dyn Embedder>>>);

impl EmbedderSlot {
    pub fn ready(embedder: Arc<dyn Embedder>) -> Self {
        Self(Arc::new(OnceLock::from(embedder)))
    }

    pub fn fill<E: Embedder + 'static>(&self, make: impl FnOnce() -> E) -> Arc<dyn Embedder> {
        Arc::clone(
            self.0
                .get_or_init(|| -> Arc<dyn Embedder> { Arc::new(make()) }),
        )
    }

    pub fn get(&self) -> Option<Arc<dyn Embedder>> {
        self.0.get().map(Arc::clone)
    }
}

impl fmt::Debug for EmbedderSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EmbedderSlot")
            .field("filled", &self.0.get().is_some())
            .finish_non_exhaustive()
    }
}
