//! The document as ingestion sees it, between a connector and the pipeline.

use crate::principal::Subject;
use chrono::{DateTime, NaiveDate, Utc};
use sha2::{Digest, Sha256};

/// One source document, normalised to text, with its resolved grants.
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
    pub grants: Vec<Subject>,
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
/// including documents skipped as unchanged — so the writer can prune what
/// disappeared. `complete` says whether that list is authoritative; an
/// incremental Jira sync only sees updated issues and must not prune.
#[derive(Debug, Default)]
pub struct FetchOutcome {
    pub documents: Vec<SourceDocument>,
    pub unchanged: Vec<UnchangedDocument>,
    pub current_refs: Vec<String>,
    pub complete: bool,
}

/// A document the connector saw but did not re-read, whose grants may still
/// have changed upstream.
#[derive(Debug, Clone)]
pub struct UnchangedDocument {
    pub doc_id: String,
    pub grants: Option<Vec<Subject>>,
}

#[must_use]
pub fn content_hash(body: &str) -> String {
    let digest = Sha256::digest(body.as_bytes());
    format!("{digest:x}")
}
