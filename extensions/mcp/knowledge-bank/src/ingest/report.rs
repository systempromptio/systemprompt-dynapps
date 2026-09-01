//! What a run is asked to do and what it reports back.

/// What one run should do.
///
/// `rebuild` ignores the caller's cursor and re-chunks every document it
/// fetches; cached embeddings and dates are still reused. It never deletes
/// another user's documents — nothing in a per-user run may.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunOptions {
    pub rebuild: bool,
    pub sources: Vec<String>,
}

/// What one run did to one source.
#[derive(Debug, Clone, Default)]
pub struct SourceReport {
    pub source_type: String,
    pub seen: usize,
    pub written: usize,
    pub unchanged: usize,
    pub failed: usize,
    pub pruned: usize,
    pub chunks: usize,
    pub error: Option<String>,
}

/// What one run did overall.
#[derive(Debug, Clone, Default)]
pub struct IngestReport {
    pub sources: Vec<SourceReport>,
    pub version: Option<String>,
    pub duration_ms: u64,
}

impl IngestReport {
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.sources.iter().any(|s| s.error.is_some())
    }
}
