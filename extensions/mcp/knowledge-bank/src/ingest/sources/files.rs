//! Files connector: a directory tree per category.
//!
//! Files connector: a directory tree per category — hand-authored meeting
//! notes, ADRs, spec trees — read straight from disk. Markdown and text are
//! indexed as-is; PDF and DOCX are extracted; anything else is skipped, never
//! fatal. Documents inherit the category's configured grants.

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
    let root = PathBuf::from(request.source.path.as_deref().unwrap_or_default());
    let mut outcome = FetchOutcome {
        complete: true,
        ..FetchOutcome::default()
    };
    if !root.is_dir() {
        tracing::warn!(path = %root.display(), source = request.source_type, "source directory missing");
        return Ok(outcome);
    }
    let include: Vec<glob::Pattern> = patterns(&request.source.include);
    let exclude: Vec<glob::Pattern> = patterns(&request.source.exclude);
    let dewrap = request.source.strategy == ChunkStrategy::Llm;

    for path in walk(&root) {
        let Ok(relative) = path.strip_prefix(&root) else {
            continue;
        };
        let rel = relative.to_string_lossy().replace('\\', "/");
        if !is_supported(&path) || !matches(&rel, &include, &exclude) {
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
        if body.trim().is_empty() {
            continue;
        }
        let body = if dewrap && path.extension().is_some_and(|e| e == "pdf") {
            dewrap_transcript(&body)
        } else {
            body
        };
        let mtime = mtime(&path).await;
        let fallback_title = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        outcome.current_refs.push(rel.clone());
        outcome.documents.push(SourceDocument {
            doc_id: SourceDocument::doc_id_for(request.source_type, &rel),
            source_type: request.source_type.to_owned(),
            source: SOURCE.to_owned(),
            source_ref: rel.clone(),
            title: infer_title(&body, &fallback_title),
            uri: format!("{}/{rel}", root.to_string_lossy().trim_end_matches('/')),
            content_hash: content_hash(&body),
            body,
            mtime,
            authored_date: None,
            grants: request.grants.unrestricted(),
        });
    }
    Ok(outcome)
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

async fn mtime(path: &Path) -> Option<DateTime<Utc>> {
    let metadata = tokio::fs::metadata(path).await.ok()?;
    let modified = metadata.modified().ok()?;
    Some(DateTime::<Utc>::from(modified))
}
