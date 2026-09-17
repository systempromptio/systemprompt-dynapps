//! The History view: one row per version, newest first, with what changed
//! since the version before it and how it performed in the window.

use serde::Serialize;

use super::diff::{ManifestDiff, diff};
use crate::handlers::ssr::format::{format_cost, format_duration_ms};
use crate::handlers::ssr::scorecard::{ScorecardView, Scores};
use crate::repositories::analysis::marketplace_versions::MarketplaceVersionMetricsRow;
use crate::repositories::evaluation::version_scores::VersionScoreRow;

pub(crate) fn short(value: &str) -> String {
    value.chars().take(12).collect()
}

pub(crate) fn latency(micros: Option<i64>, measured: i64) -> String {
    match micros {
        Some(m) if measured > 0 => format_duration_ms(m / 1000),
        _ => "—".to_owned(),
    }
}

pub(crate) fn cost(micros: i64) -> String {
    if micros > 0 {
        format_cost(micros)
    } else {
        "—".to_owned()
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VersionView {
    pub hash: String,
    pub hash_short: String,
    pub anchor: String,
    pub name: Option<String>,
    pub declared_version: Option<String>,
    pub source: String,
    pub source_hash: Option<String>,
    pub source_hash_short: Option<String>,
    pub legacy: bool,
    pub has_manifest: bool,
    pub is_current: bool,
    pub first_seen_at: String,
    pub last_seen_at: String,
    pub effective_until: Option<String>,
    pub plugin_count: i32,
    pub skill_count: i32,
    pub diff: Option<ManifestDiff>,
    pub invocations: i64,
    pub failed_invocations: i64,
    pub users: i64,
    pub conversations: i64,
    pub requests: i64,
    pub failed: i64,
    pub tokens: i64,
    pub cost: String,
    pub p50: String,
    pub p95: String,
    pub scorecard: ScorecardView,
    pub scored_sessions: i64,
    pub compare_href: Option<String>,
}

// Why: the judge's means for a version, or an unscored card when no judged
// session invoked a skill served by that hash.
fn scorecard_for(scores: &[VersionScoreRow], hash: &str) -> (ScorecardView, i64) {
    let row = scores.iter().find(|s| s.content_hash == hash);
    let scored = row.map_or(0, |r| r.scored_sessions);
    (
        ScorecardView::new(
            row.map_or_else(Scores::default, |r| Scores {
                intent: r.intent,
                quality: r.quality,
                speed: r.speed,
                cost: r.cost,
            }),
            None,
            row.map_or_else(
                || "No judged session invoked a skill served by this version".to_owned(),
                |r| {
                    format!(
                        "Mean judge scores over {} judged sessions served this version",
                        r.scored_sessions
                    )
                },
            ),
        ),
        scored,
    )
}

// Why: rows arrive newest first; the diff for a row is against the row
// *after* it in that order, which is the version it replaced. The oldest
// row diffs against nothing and reads as the first version.
pub(crate) fn views(
    rows: &[MarketplaceVersionMetricsRow],
    scores: &[VersionScoreRow],
) -> Vec<VersionView> {
    rows.iter()
        .enumerate()
        .map(|(i, row)| {
            let previous = rows.get(i + 1);
            let (scorecard, scored_sessions) = scorecard_for(scores, &row.content_hash);
            let diff = row.manifest.as_ref().map(|m| {
                diff(
                    previous.and_then(|p| p.manifest.as_ref()).map(|m| &m.0),
                    &m.0,
                )
            });
            VersionView {
                hash_short: short(&row.content_hash),
                anchor: format!("v-{}", short(&row.content_hash)),
                name: row.manifest.as_ref().map(|m| m.0.name.clone()),
                declared_version: row.manifest.as_ref().map(|m| m.0.version.clone()),
                source: row.source.clone(),
                source_hash_short: row.source_hash.as_deref().map(short),
                source_hash: row.source_hash.clone(),
                legacy: row.origin == "legacy_source_hash",
                has_manifest: row.manifest.is_some(),
                is_current: row.effective_until.is_none(),
                first_seen_at: row.first_seen_at.to_rfc3339(),
                last_seen_at: row.last_seen_at.to_rfc3339(),
                effective_until: row.effective_until.map(|t| t.to_rfc3339()),
                plugin_count: row.plugin_count,
                skill_count: row.skill_count,
                diff,
                invocations: row.invocations,
                failed_invocations: row.failed_invocations,
                users: row.users,
                conversations: row.conversations,
                requests: row.requests,
                failed: row.failed,
                tokens: row.tokens,
                cost: cost(row.cost),
                p50: latency(row.p50_micros, row.latency_measured),
                p95: latency(row.p95_micros, row.latency_measured),
                scorecard,
                scored_sessions,
                compare_href: previous.map(|p| {
                    format!(
                        "?tab=compare&a={}&b={}",
                        urlencoding::encode(&p.content_hash),
                        urlencoding::encode(&row.content_hash)
                    )
                }),
                hash: row.content_hash.clone(),
            }
        })
        .collect()
}
