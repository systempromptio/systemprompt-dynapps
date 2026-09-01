//! The document as ingestion sees it, between a connector and the pipeline.

use chrono::{DateTime, NaiveDate, Utc};
use sha2::{Digest, Sha256};

/// One source document, normalised to text.
///
/// It carries no grants: whoever fetched it could read it, and the ingestor
/// grants it to that caller.
#[derive(Debug, Clone)]
pub struct SourceDocument {
    pub doc_id: String,
    pub source_type: String,
    pub source: String,
    pub source_ref: String,
    pub title: String,
    pub uri: String,
    pub body: String,
    pub content_hash: String,
    pub mtime: Option<DateTime<Utc>>,
    pub authored_date: Option<NaiveDate>,
}

impl SourceDocument {
    #[must_use]
    pub fn doc_id_for(source_type: &str, source_ref: &str) -> String {
        format!("{source_type}:{source_ref}")
    }
}

/// What a connector produced for one source category.
///
/// `current_refs` lists every `source_ref` that exists upstream right now —
/// including documents skipped as unchanged — so the caller's grants on what
/// disappeared can be dropped. `complete` says whether that list is
/// authoritative; an incremental Jira sync only sees updated issues and must
/// not drop anything.
#[derive(Debug, Default)]
pub struct FetchOutcome {
    pub documents: Vec<SourceDocument>,
    pub unchanged: Vec<String>,
    pub current_refs: Vec<String>,
    pub complete: bool,
}

#[must_use]
pub fn content_hash(body: &str) -> String {
    let digest = Sha256::digest(body.as_bytes());
    format!("{digest:x}")
}
