//! Files connector: a directory tree per category.
//!
//! Hand-authored meeting notes, ADRs and spec trees, read straight from
//! disk under `files.root/<category>` (or the category's own `path`).
//! Markdown and text are indexed as-is; PDF and DOCX are extracted; anything
//! else is skipped, never fatal. A file whose mtime predates the caller's
//! cursor and that the index already holds is reported unchanged. The
//! listing is always complete, so a file that vanished leaves the index on
//! the next sync (FR-ING-7). Grants are not decided here: the ingestor
//! applies the category's configured subjects.

use super::FetchRequest;
use crate::config::ChunkStrategy;
use crate::error::KnowledgeBankError;
use crate::ingest::chunker::infer_title;
use crate::ingest::document::{FetchOutcome, SourceDocument, content_hash};
use crate::ingest::parsers::{dewrap_transcript, is_supported, parse_bytes};
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};

pub const SOURCE: &str = "files";

pub async fn fetch(request: &FetchRequest<'_>) -> Result<FetchOutcome, KnowledgeBankError> {
    let root = request
        .source
        .resolved_path(request.source_type, request.files);
    let mut outcome = FetchOutcome {
        complete: true,
        ..FetchOutcome::default()
    };
    if !root.is_dir() {
        tracing::warn!(path = %root.display(), source = request.source_type, "source directory missing");
        return Ok(outcome);
    }
    let include = patterns(&request.source.include);
    let exclude = patterns(&request.source.exclude);

    for path in walk(&root) {
        let Ok(relative) = path.strip_prefix(&root) else {
            continue;
        };
        let rel = relative.to_string_lossy().replace('\\', "/");
        if !is_supported(&path) || !matches(&rel, &include, &exclude) {
            continue;
        }
        let doc_id = SourceDocument::doc_id_for(request.source_type, &rel);
        let mtime = mtime(&path).await;
        outcome.current_refs.push(rel.clone());
        if request.can_skip(&doc_id, mtime) {
            outcome.unchanged.push(doc_id);
            continue;
        }
        let bytes = match tokio::fs::read(&path).await {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "unreadable file skipped");
                continue;
            },
        };
        let parsed = {
            let path = path.clone();
            tokio::task::spawn_blocking(move || parse_bytes(&path, &bytes))
                .await
                // Why: JoinError is variant-less; a panicking parser is a
                // per-document failure, not a run failure.
                // lint-ok: error-adapt
                .map_err(|e| KnowledgeBankError::Internal(format!("parser task: {e}")))
        };
        let body = match parsed {
            Ok(Ok(text)) => text,
            Ok(Err(e)) | Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "file could not be parsed");
                continue;
            },
        };
        let Some(document) = document_from_text(&DocumentSpec {
            source_type: request.source_type,
            root: &root,
            rel: &rel,
            strategy: request.source.strategy,
            body,
            mtime,
        }) else {
            continue;
        };
        outcome.documents.push(document);
    }
    Ok(outcome)
}

/// What [`document_from_text`] needs to shape one file into a document.
#[derive(Debug)]
pub struct DocumentSpec<'a> {
    pub source_type: &'a str,
    pub root: &'a Path,
    pub rel: &'a str,
    pub strategy: ChunkStrategy,
    pub body: String,
    pub mtime: Option<DateTime<Utc>>,
}

// Why: shared by the directory walk and the upload tool so both produce the
// same `doc_id`, `uri` and title for the same file. `None` when the text is
// blank.
#[must_use]
pub fn document_from_text(spec: &DocumentSpec<'_>) -> Option<SourceDocument> {
    if spec.body.trim().is_empty() {
        return None;
    }
    let dewrap =
        spec.strategy == ChunkStrategy::Llm && spec.rel.to_ascii_lowercase().ends_with(".pdf");
    let body = if dewrap {
        dewrap_transcript(&spec.body)
    } else {
        spec.body.clone()
    };
    let fallback_title = Path::new(spec.rel)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    Some(SourceDocument {
        doc_id: SourceDocument::doc_id_for(spec.source_type, spec.rel),
        source_type: spec.source_type.to_owned(),
        source: SOURCE.to_owned(),
        source_ref: spec.rel.to_owned(),
        title: infer_title(&body, &fallback_title),
        uri: format!(
            "{}/{}",
            spec.root.to_string_lossy().trim_end_matches('/'),
            spec.rel
        ),
        content_hash: content_hash(&body),
        body,
        mtime: spec.mtime,
        authored_date: None,
    })
}

fn patterns(globs: &[String]) -> Vec<glob::Pattern> {
    globs
        .iter()
        .filter_map(|g| glob::Pattern::new(g).ok())
        .collect()
}

fn matches(rel: &str, include: &[glob::Pattern], exclude: &[glob::Pattern]) -> bool {
    let included = include.is_empty() || include.iter().any(|p| p.matches(rel));
    let excluded = exclude.iter().any(|p| p.matches(rel));
    included && !excluded
}

fn walk(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut entries: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| {
                !p.file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with('.'))
            })
            .collect();
        entries.sort();
        for entry in entries {
            if entry.is_dir() {
                stack.push(entry);
            } else {
                out.push(entry);
            }
        }
    }
    out.sort();
    out
}

pub async fn mtime(path: &Path) -> Option<DateTime<Utc>> {
    let metadata = tokio::fs::metadata(path).await.ok()?;
    let modified = metadata.modified().ok()?;
    Some(DateTime::<Utc>::from(modified))
}
