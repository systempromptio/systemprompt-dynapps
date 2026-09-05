//! Assembling the `/admin/mcp` list: joining declaration to runtime, deriving
//! status, and ordering.
//!
//! The join is a full outer one in spirit. Every declared server appears even
//! with no traffic, and every server the runtime tables name appears even when
//! nothing declares it — the second case is an operator problem (a binary
//! serving under a name the catalog does not know) and hiding it would be the
//! one failure this page must not have.

use std::collections::HashMap;

use crate::handlers::catalog::sorting::{SortColumn, apply_direction};
use crate::handlers::ssr::format::{format_duration_ms, short_num};
use crate::repositories::mcp::runtime::{McpServerActivity, McpServerLiveness};
use crate::types::McpServerDetail;

use super::view::{McpKpiView, McpServerRow};

pub(super) const BASE_URL: &str = "/admin/mcp";
pub(super) const WINDOW_HOURS: i64 = 24;

/// The columns `/admin/mcp` can be ordered by.
pub(super) fn columns() -> Vec<SortColumn> {
    vec![
        SortColumn {
            key: "id",
            label: "Server",
            class: "",
            hint: "The id declared in services/mcp, or the name the runtime used",
        },
        SortColumn {
            key: "status",
            label: "Status",
            class: "",
            hint: "Alive when a session has spoken inside the heartbeat window",
        },
        SortColumn {
            key: "sessions",
            label: "Sessions",
            class: "sp-table__cell--num",
            hint: "Open sessions, and how many of them are still beating",
        },
        SortColumn {
            key: "identities",
            label: "Identities",
            class: "sp-table__cell--num",
            hint: "Unexpired proxy identities a live session is acting as",
        },
        SortColumn {
            key: "calls",
            label: "Calls 24h",
            class: "sp-table__cell--num",
            hint: "Tool executions in the last 24 hours, against the 24 before",
        },
        SortColumn {
            key: "errors",
            label: "Error rate",
            class: "sp-table__cell--num",
            hint: "Failed and timed-out calls as a share of the window",
        },
        SortColumn {
            key: "p95",
            label: "p95",
            class: "sp-table__cell--num",
            hint: "95th percentile execution time in the window",
        },
        SortColumn {
            key: "users",
            label: "Users",
            class: "sp-table__cell--num",
            hint: "Distinct callers in the window",
        },
        SortColumn {
            key: "last",
            label: "Last call",
            class: "sp-table__cell--date",
            hint: "When this server last executed a tool",
        },
        SortColumn {
            key: "grants",
            label: "Grants",
            class: "sp-table__cell--num",
            hint: "Access-control rules naming this server",
        },
    ]
}

/// The runtime facts for one server, keyed by the name the runtime used.
pub(super) struct Runtime {
    pub liveness: HashMap<String, McpServerLiveness>,
    pub activity: HashMap<String, McpServerActivity>,
}

impl Runtime {
    pub(super) fn new(liveness: Vec<McpServerLiveness>, activity: Vec<McpServerActivity>) -> Self {
        Self {
            liveness: liveness
                .into_iter()
                .map(|l| (l.server_id.clone(), l))
                .collect(),
            activity: activity
                .into_iter()
                .map(|a| (a.server_name.clone(), a))
                .collect(),
        }
    }

    /// Every name either runtime table knows, so an undeclared server is still
    /// listed rather than silently dropped.
    pub(super) fn names(&self) -> Vec<String> {
        let mut out: Vec<String> = self
            .liveness
            .keys()
            .chain(self.activity.keys())
            .cloned()
            .collect();
        out.sort();
        out.dedup();
        out
    }
}

pub fn status_of(
    configured: bool,
    enabled: bool,
    liveness: Option<&McpServerLiveness>,
) -> (&'static str, &'static str) {
    if !configured {
        return ("Unconfigured", "warn");
    }
    if !enabled {
        return ("Disabled", "muted");
    }
    match liveness {
        Some(l) if l.is_alive() => ("Alive", "ok"),
        Some(_) => ("Stale", "warn"),
        None => ("Never connected", "muted"),
    }
}

fn error_rate(calls: i64, errors: i64) -> (String, &'static str) {
    if calls == 0 {
        return ("\u{2014}".to_owned(), "muted");
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "call counts are far below the f64 mantissa; this is a percentage for display"
    )]
    let pct = (errors as f64 / calls as f64) * 100.0;
    let tone = if pct >= 10.0 {
        "err"
    } else if pct > 0.0 {
        "warn"
    } else {
        "ok"
    };
    (format!("{pct:.1}%"), tone)
}

fn delta(calls: i64, prior: i64) -> (String, &'static str) {
    if prior == 0 {
        return (String::new(), "");
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "call counts are far below the f64 mantissa; this is a percentage for display"
    )]
    let pct = ((calls - prior) as f64 / prior as f64) * 100.0;
    if pct.abs() < 0.5 {
        return ("0%".to_owned(), "");
    }
    let dir = if pct > 0.0 { "up" } else { "down" };
    (format!("{pct:+.0}%"), dir)
}

fn ms_display(ms: Option<f64>) -> String {
    ms.map_or_else(
        || "\u{2014}".to_owned(),
        |v| {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "a millisecond figure is rendered to whole milliseconds"
            )]
            format_duration_ms(v.round() as i64)
        },
    )
}

fn when_display(t: Option<chrono::DateTime<chrono::Utc>>) -> String {
    t.map_or_else(
        || "\u{2014}".to_owned(),
        crate::handlers::ssr::format::local_time,
    )
}

fn auth_label(server: Option<&McpServerDetail>) -> String {
    let Some(server) = server else {
        return "unknown".to_owned();
    };
    if !server.oauth_required {
        return "open".to_owned();
    }
    let audience = if server.oauth_audience.is_empty() {
        "any"
    } else {
        server.oauth_audience.as_str()
    };
    if server.oauth_scopes.is_empty() {
        return format!("aud {audience}");
    }
    format!("aud {audience} / {}", server.oauth_scopes.join(" "))
}

fn transport_of(server: Option<&McpServerDetail>) -> String {
    match server {
        None => "\u{2014}".to_owned(),
        Some(s) if !s.endpoint.is_empty() => s.endpoint.clone(),
        Some(s) => format!("localhost:{}", s.port),
    }
}

pub(super) struct RowInputs<'a> {
    pub id: &'a str,
    pub server: Option<&'a McpServerDetail>,
    pub runtime: &'a Runtime,
    pub plugin_count: usize,
    pub assignment_count: i64,
}

pub(super) fn build_row(input: &RowInputs<'_>) -> McpServerRow {
    let liveness = input.runtime.liveness.get(input.id);
    let activity = input.runtime.activity.get(input.id);
    let configured = input.server.is_some();
    let enabled = input.server.is_some_and(|s| s.enabled);
    let (status_label, status_tone) = status_of(configured, enabled, liveness);

    let calls = activity.map_or(0, |a| a.calls);
    let errors = activity.map_or(0, |a| a.failures + a.timeouts);
    let (error_rate_display, error_tone) = error_rate(calls, errors);
    let (delta_display, delta_dir) = delta(calls, activity.map_or(0, |a| a.prior_calls));

    McpServerRow {
        id: input.id.to_owned(),
        description: input
            .server
            .map(|s| s.description.clone())
            .filter(|d| !d.is_empty())
            .unwrap_or_else(|| "Not declared in services/mcp.".to_owned()),
        detail_url: format!("{BASE_URL}/{}", input.id),
        matrix_url: super::super::view::matrix_url(
            crate::types::ENTITY_MCP_SERVER,
            input.id,
        ),
        source_path: input
            .server
            .map(|s| s.source_path.clone())
            .unwrap_or_default(),
        configured,
        enabled,
        status_label,
        status_tone,
        server_type: input
            .server
            .map_or_else(|| "\u{2014}".to_owned(), |s| s.server_type.clone()),
        transport: transport_of(input.server),
        auth_label: auth_label(input.server),
        oauth_required: input.server.is_some_and(|s| s.oauth_required),
        sessions_open: liveness.map_or(0, |l| l.sessions_open),
        sessions_alive: liveness.map_or(0, |l| l.sessions_alive),
        proxy_identities: liveness.map_or(0, |l| l.proxy_identities),
        calls,
        errors,
        error_rate_display,
        error_tone,
        p95_display: ms_display(activity.and_then(|a| a.p95_ms)),
        distinct_users: activity.map_or(0, |a| a.distinct_users),
        delta_display,
        delta_dir,
        last_call_display: when_display(activity.and_then(|a| a.last_call_at)),
        plugin_count: input.plugin_count,
        assignment_count: input.assignment_count,
    }
}

/// Order the assembled rows by the requested column.
pub(super) fn sort_rows(rows: &mut [McpServerRow], key: &str, dir: &str) {
    match key {
        "status" => apply_direction(rows, dir, |a, b| {
            a.sessions_alive
                .cmp(&b.sessions_alive)
                .then_with(|| a.id.cmp(&b.id))
        }),
        "sessions" => apply_direction(rows, dir, |a, b| a.sessions_open.cmp(&b.sessions_open)),
        "identities" => {
            apply_direction(rows, dir, |a, b| a.proxy_identities.cmp(&b.proxy_identities));
        },
        "errors" => apply_direction(rows, dir, |a, b| a.errors.cmp(&b.errors)),
        "p95" => apply_direction(rows, dir, |a, b| a.p95_display.cmp(&b.p95_display)),
        "users" => apply_direction(rows, dir, |a, b| a.distinct_users.cmp(&b.distinct_users)),
        "last" => apply_direction(rows, dir, |a, b| {
            a.last_call_display.cmp(&b.last_call_display)
        }),
        "grants" => apply_direction(rows, dir, |a, b| a.assignment_count.cmp(&b.assignment_count)),
        "id" => apply_direction(rows, dir, |a, b| a.id.cmp(&b.id)),
        _ => apply_direction(rows, dir, |a, b| a.calls.cmp(&b.calls)),
    }
}

/// The five headline facts, plus the two that only matter when they are wrong.
pub(super) fn kpis(rows: &[McpServerRow]) -> Vec<McpKpiView> {
    let configured = rows.iter().filter(|r| r.configured).count();
    let alive = rows.iter().filter(|r| r.sessions_alive > 0).count();
    let sessions: i64 = rows.iter().map(|r| r.sessions_open).sum();
    let identities: i64 = rows.iter().map(|r| r.proxy_identities).sum();
    let calls: i64 = rows.iter().map(|r| r.calls).sum();
    let errors: i64 = rows.iter().map(|r| r.errors).sum();
    let unconfigured = rows.len() - configured;
    let (rate, rate_tone) = error_rate(calls, errors);

    vec![
        McpKpiView {
            label: "Declared",
            value: configured.to_string(),
            sub: format!("{unconfigured} serving undeclared"),
            tone: if unconfigured > 0 { "warn" } else { "" },
            unit: "",
        },
        McpKpiView {
            label: "Alive now",
            value: alive.to_string(),
            sub: format!("of {configured} declared"),
            tone: if alive == 0 { "warn" } else { "ok" },
            unit: "",
        },
        McpKpiView {
            label: "Open sessions",
            value: short_num(sessions),
            sub: format!("{identities} proxy identities"),
            tone: "",
            unit: "",
        },
        McpKpiView {
            label: "Calls 24h",
            value: short_num(calls),
            sub: "tool executions".to_owned(),
            tone: "",
            unit: "",
        },
        McpKpiView {
            label: "Errors 24h",
            value: short_num(errors),
            sub: "failed or timed out".to_owned(),
            tone: if errors > 0 { "err" } else { "ok" },
            unit: "",
        },
        McpKpiView {
            label: "Error rate",
            value: rate,
            sub: "of calls in the window".to_owned(),
            tone: rate_tone,
            unit: "",
        },
    ]
}
