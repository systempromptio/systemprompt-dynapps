//! Rendering of retrieval results, source listings, statistics, sync and
//! prune reports into the [`CliArtifact`] every tool returns. Each hit is
//! rendered with the fields the agent policy ranks by — category, date, date
//! source, and the `uri#lines` pointer — so a client never has to parse the
//! snippet to find them.

use crate::ingest::UploadReport;
use crate::maintenance::PruneReport;
use crate::retriever::{IndexStats, KnowledgeHit, KnowledgeSource};
use systemprompt::models::artifacts::{CliArtifact, TextArtifact};

pub(super) fn format_hits(hits: &[KnowledgeHit], query: &str) -> (CliArtifact, String) {
    let body = if hits.is_empty() {
        format!(
            "No results for `{query}` in the searched categories. If you have not synced yet, \
             run sync_knowledge with your Atlassian credentials on the connection."
        )
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
                .map_or_else(String::new, |n| format!(", {n} docs readable by you"));
            let synced = s.last_synced.as_deref().map_or_else(
                || ", you have not synced it".to_owned(),
                |t| format!(", you synced {t}"),
            );
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
        "documents: {}\nchunks: {}\ngrants: {}\nsyncing_users: {}\nembedding_cache_rows: \
         {}\ndatabase_mib: {:.1}\nlast_built: {}\nversion: {}",
        stats.documents,
        stats.chunks,
        stats.grants,
        stats.syncing_users,
        stats.embedding_cache_rows,
        stats.database_bytes as f64 / (1024.0 * 1024.0),
        stats.last_built.as_deref().unwrap_or("never"),
        stats.version.as_deref().unwrap_or("unknown")
    );
    (
        CliArtifact::text(TextArtifact::new(&body).with_title("Knowledge Index Stats")),
        format!(
            "{} documents, {} chunks, {} users syncing",
            stats.documents, stats.chunks, stats.syncing_users
        ),
    )
}

pub(super) fn format_sync_started(
    sources: &[String],
    rebuild: bool,
    with_atlassian: bool,
) -> (CliArtifact, String) {
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
    let reading = if with_atlassian {
        "reading Atlassian as you"
    } else {
        "files categories only, no Atlassian credentials on this connection"
    };
    let summary = format!(
        "Your knowledge {mode} started for {scope}, {reading}. Follow progress with \
         list_knowledge_sources."
    );
    (
        CliArtifact::text(TextArtifact::new(&summary).with_title("Knowledge Sync")),
        summary,
    )
}

pub(super) fn format_prune_report(report: &PruneReport) -> (CliArtifact, String) {
    let verb = if report.dry_run {
        "would remove"
    } else {
        "removed"
    };
    let body = format!(
        "mode: {}\nstale users ({}): {}\nmissing users ({}): {}\ngrants {verb}: {}\nsync states \
         {verb}: {}\ndocuments {verb}: {}\ndate-cache rows {verb}: {}\nembedding-cache rows \
         {verb}: {}",
        if report.dry_run { "dry run" } else { "applied" },
        report.stale_users.len(),
        list_or_none(&report.stale_users),
        report.missing_users.len(),
        list_or_none(&report.missing_users),
        report.grants_removed,
        report.sync_states_removed,
        report.documents_deleted,
        report.date_cache_removed,
        report.embedding_cache_removed,
    );
    (
        CliArtifact::text(TextArtifact::new(&body).with_title("Knowledge Prune")),
        format!(
            "{verb} {} grant(s) and {} document(s)",
            report.grants_removed, report.documents_deleted
        ),
    )
}

pub(super) fn format_upload_report(report: &UploadReport) -> (CliArtifact, String) {
    let readers = report
        .granted_to
        .iter()
        .map(|s| format!("{}:{}", s.kind, s.value))
        .collect::<Vec<_>>()
        .join(", ");
    let body = format!(
        "{}\ndoc_id: {}\nuri: {}\nchunks: {}\nreadable by: {readers}\nindex version: {}",
        if report.replaced {
            "Replaced and re-indexed."
        } else {
            "Added and indexed."
        },
        report.doc_id,
        report.uri,
        report.chunks,
        report.version,
    );
    (
        CliArtifact::text(TextArtifact::new(&body).with_title("Knowledge Document Added")),
        format!(
            "{} '{}' ({} chunk(s))",
            if report.replaced { "replaced" } else { "added" },
            report.title,
            report.chunks
        ),
    )
}

fn list_or_none(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_owned()
    } else {
        items.join(", ")
    }
}
