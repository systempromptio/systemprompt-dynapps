//! Presentation shaping for the governance warnings page.
//!
//! The grouping arithmetic itself lives in
//! [`crate::repositories::governance::warnings`], where the tests can reach
//! it; this module only turns those rows into strings and URLs.

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use serde::Serialize;
use systemprompt::identifiers::{TraceId, UserId};

use super::BASE_URL;
use crate::handlers::ssr::entity_urls::trace_detail_url;
use crate::handlers::ssr::list_view::TimeRangeContext;
use crate::repositories::governance::warnings::{
    SafetyFindingRollupRow, WarningDecisionRow, WarningGroup, WarningGroupBy, WarningRollupRow,
    group_warning_rollup,
};
use crate::util::time_range::TimeRange;

#[derive(Debug, Serialize)]
pub(super) struct GroupRow {
    pub(super) group: String,
    pub(super) warnings: i64,
    pub(super) tools: i64,
    pub(super) users: i64,
    pub(super) first_seen: String,
    pub(super) last_seen: String,
    pub(super) example_reason: String,
}

#[derive(Debug, Serialize)]
pub(super) struct RecentRow {
    pub(super) decision_url: String,
    pub(super) policy: String,
    pub(super) tool_name: String,
    pub(super) user_id: UserId,
    pub(super) reason: String,
    pub(super) trace_url: Option<String>,
    pub(super) created_at: String,
}

#[derive(Debug, Serialize)]
pub(super) struct FindingRow {
    pub(super) category: String,
    pub(super) scanner: String,
    pub(super) severity: String,
    pub(super) phase: String,
    pub(super) findings: i64,
    pub(super) blocked: i64,
    pub(super) absorbed: i64,
    pub(super) last_seen: String,
}

pub(super) fn group_rows(rows: &[WarningRollupRow], by: WarningGroupBy) -> Vec<GroupRow> {
    group_warning_rollup(rows, by)
        .into_iter()
        .map(|g: WarningGroup| GroupRow {
            group: g.group,
            warnings: g.warnings,
            tools: g.tools,
            users: g.users,
            first_seen: stamp(g.first_seen),
            last_seen: stamp(g.last_seen),
            example_reason: truncate(&g.example_reason, 160),
        })
        .collect()
}

pub(super) fn recent_rows(rows: &[WarningDecisionRow]) -> Vec<RecentRow> {
    rows.iter()
        .map(|r| RecentRow {
            decision_url: format!("/admin/governance/decisions/{}", r.id),
            policy: r.policy.clone(),
            tool_name: r.tool_name.clone(),
            user_id: r.user_id.clone(),
            reason: truncate(&r.reason, 200),
            trace_url: r
                .trace_id
                .as_ref()
                .filter(|t| !t.is_empty())
                .map(|t| trace_detail_url(&TraceId::new(t.clone()))),
            created_at: local_stamp(r.created_at),
        })
        .collect()
}

pub(super) fn finding_rows(rows: &[SafetyFindingRollupRow]) -> Vec<FindingRow> {
    rows.iter()
        .map(|r| FindingRow {
            category: r.category.clone(),
            scanner: r.scanner.clone(),
            severity: r.severity.clone(),
            phase: r.phase.clone(),
            findings: r.count,
            blocked: r.blocked_count,
            absorbed: r.count - r.blocked_count,
            last_seen: local_stamp(r.last_seen),
        })
        .collect()
}

pub(super) fn distinct<T: Ord>(values: impl Iterator<Item = T>) -> i64 {
    i64::try_from(values.collect::<BTreeSet<_>>().len()).unwrap_or(i64::MAX)
}

pub(super) fn time_range_context(
    range: TimeRange,
    preset: Option<&str>,
    by: WarningGroupBy,
) -> TimeRangeContext {
    TimeRangeContext {
        preset: preset.unwrap_or("24h").to_owned(),
        from: range.from.to_rfc3339(),
        to: range.to.to_rfc3339(),
        base_url: BASE_URL,
        query: match by {
            WarningGroupBy::Policy => "&group_by=policy",
            WarningGroupBy::Tool => "&group_by=tool",
            WarningGroupBy::User => "&group_by=user",
        },
    }
}

pub(super) fn page_url(range: TimeRange, preset: Option<&str>, by: WarningGroupBy) -> String {
    format!(
        "{BASE_URL}?{}&group_by={}",
        window_query(range, preset),
        by.as_str()
    )
}

pub(super) fn csv_url(range: TimeRange, by: WarningGroupBy) -> String {
    format!(
        "{BASE_URL}.csv?from={}&to={}&group_by={}",
        urlencoding::encode(&range.from.to_rfc3339()),
        urlencoding::encode(&range.to.to_rfc3339()),
        by.as_str()
    )
}

// Why: a named preset travels as the preset, not as its resolved bounds. The
// bounds are recomputed from `now` on every request, so pinning them into the
// link would freeze a "last 24 hours" view at the hour it was rendered.
fn window_query(range: TimeRange, preset: Option<&str>) -> String {
    match preset {
        Some(p) if p != "custom" => format!("preset={p}"),
        _ => format!(
            "from={}&to={}",
            urlencoding::encode(&range.from.to_rfc3339()),
            urlencoding::encode(&range.to.to_rfc3339())
        ),
    }
}

fn stamp(value: Option<DateTime<Utc>>) -> String {
    value.map_or_else(String::new, local_stamp)
}

fn local_stamp(value: DateTime<Utc>) -> String {
    value
        .with_timezone(&chrono::Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let head: String = text.chars().take(max.saturating_sub(1)).collect();
    format!("{head}…")
}
