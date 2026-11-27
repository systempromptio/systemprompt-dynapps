//! What the indexing queue and the chunk table hand to the indexer.

use crate::ids::{ArticleId, SpaceId};
use crate::model::RagMode;

/// A claimed indexing job: the variant it is about, by article and language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueuedJob {
    pub id: i64,
    pub article: ArticleId,
    pub lang: String,
}

/// A variant as the indexer reads it, with where its article sits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexSource {
    pub space: SpaceId,
    pub rag_mode: RagMode,
    pub trashed: bool,
    pub title: String,
    pub description: String,
    pub body: String,
    pub content_hash: String,
}

/// A chunk to store, with the synopsis it keeps from the version of the
/// article it replaces when its text did not change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewChunk {
    pub heading_path: String,
    pub content: String,
    pub content_hash: String,
    pub synopsis: Option<String>,
}

/// A chunk still waiting for its synopsis, with the document it was cut from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynopsisTask {
    pub id: i64,
    pub lang: String,
    pub content: String,
    pub content_hash: String,
    pub body: String,
}

/// A synopsis to store, bound to the text it describes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynopsisWrite {
    pub id: i64,
    pub text: String,
    pub content_hash: String,
}

/// A chunk waiting for its vector, with the document context its prefix is
/// made from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbedTask {
    pub id: i64,
    pub content: String,
    pub heading_path: String,
    pub synopsis: Option<String>,
    pub title: String,
    pub description: String,
}

/// How many chunks one embedding configuration produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FingerprintCount {
    pub fingerprint: String,
    pub chunks: i64,
}
