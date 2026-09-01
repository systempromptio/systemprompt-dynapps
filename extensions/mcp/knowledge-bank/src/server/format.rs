//! Rendering of retrieval results, source listings, statistics, and sync
//! reports into the [`CliArtifact`] every tool returns. Each hit is rendered
//! with the fields the agent policy ranks by — category, date, date source,
//! and the `uri#lines` pointer — so a client never has to parse the snippet
//! to find them.

use crate::ingest::IngestReport;
use crate::retriever::{IndexStats, KnowledgeHit, KnowledgeSource};
use systemprompt::models::artifacts::{CliArtifact, TextArtifact};

pub(super) fn not_configured_artifact(reason: &str) -> (CliArtifact, String) {
    let summary = format!("Knowledge backend not configured: {reason}");
    (
        CliArtifact::text(TextArtifact::new(&summary).with_title("Knowledge Bank")),
        summary,
    )
}

pub(super) fn format_hits(hits: &[KnowledgeHit], query: &str) -> (CliArtifact, String) {
    let body = if hits.is_empty() {
        format!("No results for `{query}` in the searched categories.")
    } else {
        hits.iter().map(format_hit).collect::<Vec<_>>().join("\n\n")
    };
    (
        CliArtifact::text(TextArtifact::new(&body).with_title("Knowledge Search Results")),
        format!("{} result(s) for `{query}`", hits.len()),
    )
}

fn format_hit(hit: &KnowledgeHit) -> String {
    let pointer = hit
        .lines
        .as_deref()
        .map_or_else(|| hit.uri.clone(), |l| format!("{}#L{l}", hit.uri));
    format!(
        "[{:.2}] {} ({} / {}) chunk {}/{}\ndate: {} ({})\n{}\n{pointer}",
        hit.score,
        hit.title,
        hit.source_type,
        hit.source,
        hit.chunk_index + 1,
        hit.chunk_total,
        hit.date.as_deref().unwrap_or("unknown"),
        hit.date_source.as_deref().unwrap_or("none"),
        hit.snippet,
    )
}

pub(super) fn format_sources(sources: &[KnowledgeSource]) -> (CliArtifact, String) {
    if sources.is_empty() {
        let summary = "No knowledge sources are configured on this instance.".to_owned();
        return (
            CliArtifact::text(TextArtifact::new(&summary).with_title("Knowledge Sources")),
            summary,
        );
    }
    let body = sources
        .iter()
        .map(|s| {
            let docs = s
                .doc_count
                .map_or_else(String::new, |n| format!(", {n} docs readable"));
            let synced = s
                .last_synced
                .as_deref()
                .map_or_else(|| ", never synced".to_owned(), |t| format!(", synced {t}"));
            let archived = if s.archived { ", archive (opt-in)" } else { "" };
            format!(
                "{} — {} ({}{docs}{synced}{archived})\n{}",
                s.id,
                s.name,
                if s.available { "available" } else { "empty" },
                s.description
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    (
        CliArtifact::text(TextArtifact::new(&body).with_title("Knowledge Sources")),
        format!("{} knowledge source(s)", sources.len()),
    )
}

pub(super) fn format_index_stats(stats: &IndexStats) -> (CliArtifact, String) {
    let body = format!(
        "documents: {}\nchunks: {}\nlast_built: {}\nversion: {}",
        stats.documents,
        stats.chunks,
        stats.last_built.as_deref().unwrap_or("never"),
        stats.version.as_deref().unwrap_or("unknown")
    );
    (
        CliArtifact::text(TextArtifact::new(&body).with_title("Knowledge Index Stats")),
        format!("{} documents, {} chunks", stats.documents, stats.chunks),
    )
}

pub(super) fn format_sync_started(sources: &[String], rebuild: bool) -> (CliArtifact, String) {
    let scope = if sources.is_empty() {
        "every configured source".to_owned()
    } else {
        sources.join(", ")
    };
    let mode = if rebuild {
        "rebuild"
    } else {
        "incremental sync"
    };
    let summary = format!(
        "Knowledge {mode} started for {scope}. Follow progress with knowledge_index_stats and \
         list_knowledge_sources."
    );
    (
        CliArtifact::text(TextArtifact::new(&summary).with_title("Knowledge Sync")),
        summary,
    )
}

#[must_use]
pub fn format_report(report: &IngestReport) -> String {
    let mut lines: Vec<String> = report
        .sources
        .iter()
        .map(|s| {
            format!(
                "{:<20} seen {:>5}  written {:>5}  unchanged {:>5}  failed {:>4}  pruned {:>4}  chunks {:>6}{}",
                s.source_type,
                s.seen,
                s.written,
                s.unchanged,
                s.failed,
                s.pruned,
                s.chunks,
                s.error
                    .as_deref()
                    .map_or_else(String::new, |e| format!("  ERROR {e}"))
            )
        })
        .collect();
    lines.push(format!(
        "version {}  ({} ms)",
        report.version.as_deref().unwrap_or("unknown"),
        report.duration_ms
    ));
    lines.join("\n")
}
