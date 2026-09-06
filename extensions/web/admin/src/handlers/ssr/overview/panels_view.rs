//! The three panels under the KPI strip: MCP liveness, the human queues, and
//! the two spend leaderboards.
//!
//! Each panel carries its loader's error instead of its rows when the read
//! failed, so one broken aggregate costs the reader that panel and nothing
//! else.

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::handlers::ssr::format::{format_cost, short_num};
use crate::repositories::overview::liveness::{
    HEARTBEAT_INTERVAL_SECS, McpHeartbeatRow, liveness_state,
};
use crate::repositories::overview::queues::UsageAnomalyRow;
use crate::repositories::overview::scopes::ScopeCostRow;
use crate::types::McpServerDetail;

use super::data::{OverviewData, TOP_SCOPES};

#[derive(Debug, Serialize)]
pub(super) struct McpStripView {
    pub error: Option<String>,
    pub rows: Vec<McpRowView>,
    pub has_rows: bool,
    pub count: usize,
    pub summary: String,
}

#[derive(Debug, Serialize)]
pub(super) struct McpRowView {
    pub id: String,
    pub href: String,
    pub description: String,
    pub state_label: &'static str,
    pub state_tone: &'static str,
    pub heartbeat_display: String,
    pub sessions: i64,
}

#[derive(Debug, Serialize)]
pub(super) struct QueuesView {
    pub approvals_error: Option<String>,
    pub approvals: i64,
    pub approvals_tone: &'static str,
    pub anomalies_error: Option<String>,
    pub anomalies: Vec<OverviewAnomalyRowView>,
    pub anomaly_count: usize,
    pub has_anomalies: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct OverviewAnomalyRowView {
    pub metric: &'static str,
    pub window_display: String,
    pub observed_display: String,
    pub baseline_display: String,
    pub multiple_display: String,
}

#[derive(Debug, Serialize)]
pub(super) struct ScopeBoardView {
    pub title: &'static str,
    pub caption: &'static str,
    pub all_href: &'static str,
    pub error: Option<String>,
    pub rows: Vec<ScopeRowView>,
    pub has_rows: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct ScopeRowView {
    pub label: String,
    pub href: Option<String>,
    pub requests_display: String,
    pub cost_display: String,
    pub share_display: String,
    pub is_unattributed: bool,
}

// Why: the strip lists what `services/mcp/*.yaml` declares, not what has a
// heartbeat row. A configured server nothing has ever reached is the finding
// the panel exists to surface, and it has no row to be found by.
pub(super) fn mcp_strip(data: &OverviewData) -> McpStripView {
    let heartbeats = data.heartbeats.as_deref().unwrap_or(&[]);
    let now = Utc::now();
    let rows: Vec<McpRowView> = data
        .configured
        .iter()
        .filter(|server| server.enabled)
        .map(|server| mcp_row(server, heartbeats, now))
        .collect();
    let alive = rows.iter().filter(|row| row.state_tone == "ok").count();
    McpStripView {
        error: data
            .heartbeats
            .as_ref()
            .err()
            .map(|e| format!("The session heartbeat could not be read: {e}")),
        summary: format!(
            "{alive} of {} seen within {}m",
            rows.len(),
            HEARTBEAT_INTERVAL_SECS * 2 / 60,
        ),
        count: rows.len(),
        has_rows: !rows.is_empty(),
        rows,
    }
}

fn mcp_row(
    server: &McpServerDetail,
    heartbeats: &[McpHeartbeatRow],
    now: DateTime<Utc>,
) -> McpRowView {
    let id = server.id.as_str();
    let beat = heartbeats.iter().find(|row| row.server_id == id);
    let last = beat.and_then(|row| row.last_heartbeat);
    let state = liveness_state(now, last, HEARTBEAT_INTERVAL_SECS);
    McpRowView {
        id: id.to_owned(),
        href: format!("/admin/mcp/{}", urlencoding::encode(id)),
        description: server.description.clone(),
        state_label: state.label(),
        state_tone: state.tone(),
        heartbeat_display: last.map_or_else(|| "\u{2014}".to_owned(), |ts| ago_display(now, ts)),
        sessions: beat.map_or(0, |row| row.active_sessions),
    }
}

pub(super) fn queues(data: &OverviewData) -> QueuesView {
    let approvals = data.pending_approvals.as_ref().copied().unwrap_or(0);
    let anomalies: Vec<OverviewAnomalyRowView> = data
        .anomalies
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(anomaly_row)
        .collect();
    QueuesView {
        approvals_error: data
            .pending_approvals
            .as_ref()
            .err()
            .map(|e| format!("The approval queue could not be read: {e}")),
        approvals,
        approvals_tone: if approvals > 0 { "warn" } else { "ok" },
        anomalies_error: data
            .anomalies
            .as_ref()
            .err()
            .map(|e| format!("Open anomalies could not be read: {e}")),
        has_anomalies: !anomalies.is_empty(),
        anomaly_count: anomalies.len(),
        anomalies,
    }
}

fn anomaly_row(row: &UsageAnomalyRow) -> OverviewAnomalyRowView {
    let is_cost = row.metric == "cost";
    let show = |value: i64| {
        if is_cost {
            format_cost(value)
        } else {
            short_num(value)
        }
    };
    OverviewAnomalyRowView {
        metric: match row.metric.as_str() {
            "cost" => "Spend",
            "errors" => "Errors",
            _ => "Requests",
        },
        window_display: ago_display(Utc::now(), row.window_start),
        observed_display: show(row.observed),
        baseline_display: show(row.baseline),
        multiple_display: multiple_display(row.observed, row.baseline),
    }
}

fn multiple_display(observed: i64, baseline: i64) -> String {
    if baseline <= 0 {
        return "new".to_owned();
    }
    format!("{:.1}\u{d7} baseline", observed as f64 / baseline as f64)
}

#[derive(Clone, Copy)]
pub(super) struct BoardSpec {
    pub title: &'static str,
    pub caption: &'static str,
    pub all_href: &'static str,
    pub unattributed: &'static str,
}

pub(super) fn scope_board(
    spec: BoardSpec,
    loaded: &Result<Vec<ScopeCostRow>, sqlx::Error>,
) -> ScopeBoardView {
    let rows = loaded.as_deref().unwrap_or(&[]);
    let total: i64 = rows.iter().map(|row| row.cost_microdollars).sum();
    let rows: Vec<ScopeRowView> = rows
        .iter()
        .take(usize::try_from(TOP_SCOPES).unwrap_or(5))
        .map(|row| scope_row(row, total, spec.all_href, spec.unattributed))
        .collect();
    ScopeBoardView {
        title: spec.title,
        caption: spec.caption,
        all_href: spec.all_href,
        error: loaded
            .as_ref()
            .err()
            .map(|e| format!("{} could not be read: {e}", spec.title)),
        has_rows: !rows.is_empty(),
        rows,
    }
}

// Why: the unattributed bucket is a real row with no page behind it — traffic
// with no user, a job or MCP actor, or a person no primary container covers.
// It is listed rather than dropped so the board sums back to the spend tile.
fn scope_row(row: &ScopeCostRow, total: i64, base: &str, absent: &str) -> ScopeRowView {
    let unattributed = row.is_unattributed();
    ScopeRowView {
        label: if unattributed {
            absent.to_owned()
        } else {
            row.label.clone()
        },
        href: (!unattributed).then(|| format!("{base}/{}", urlencoding::encode(&row.scope_id))),
        requests_display: short_num(row.requests),
        cost_display: format_cost(row.cost_microdollars),
        share_display: share_display(row.cost_microdollars, total),
        is_unattributed: unattributed,
    }
}

fn share_display(part: i64, total: i64) -> String {
    if total <= 0 {
        return "\u{2014}".to_owned();
    }
    format!("{:.0}%", part as f64 / total as f64 * 100.0)
}

// Why: an absolute timestamp makes a reader do the subtraction the state badge
// already did. The elapsed time is the fact the row is about.
fn ago_display(now: DateTime<Utc>, ts: DateTime<Utc>) -> String {
    let secs = (now - ts).num_seconds().max(0);
    if secs < 60 {
        format!("{secs}s ago")
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86_400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86_400)
    }
}
