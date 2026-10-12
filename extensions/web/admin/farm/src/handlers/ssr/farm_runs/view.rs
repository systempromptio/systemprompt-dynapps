//! View-model assembly for the runs explorer's live half: KPI tiles that are
//! also the status filters they count, the rows, sort headers and pagination,
//! all rebuilt from the one [`RunsQuery`].

use systemprompt_web_admin_core::ssr::list_view::{
    DEFAULT_PAGE_SIZE, PageWindow, paginate_prefixed,
};
use systemprompt_web_admin_core::ssr::types::table::SortHeaderView;
use systemprompt_web_shared::format::{format_cost, format_duration_ms};

use super::context::{RunKpiView, RunSortHeaders, RunsLiveView};
use super::query::{RunSortColumn, RunsQuery};
use crate::handlers::ssr::shared::run_row_views;
use crate::repositories::rows::RunRow;
use crate::repositories::summaries::RunStats;
use crate::types::RunStatus;

pub(super) struct RunsPage<'a> {
    pub(super) rows: &'a [RunRow],
    pub(super) total: i64,
    pub(super) stats: &'a RunStats,
    pub(super) page: i64,
    pub(super) column: RunSortColumn,
}

pub(super) fn live_view(query: &RunsQuery, data: &RunsPage<'_>) -> RunsLiveView {
    let rows = run_row_views(data.rows);
    let shown = i64::try_from(rows.len()).unwrap_or(DEFAULT_PAGE_SIZE);
    let window = PageWindow::new(data.page, DEFAULT_PAGE_SIZE, data.total, shown, "runs");
    RunsLiveView {
        live_active: rows.iter().any(|r| r.is_live),
        kpis: kpis(query, data.stats),
        has_rows: !rows.is_empty(),
        rows,
        filtered: is_filtered(query),
        pagination: paginate_prefixed(window, &query.url_prefix(&["page"])),
        sort_headers: sort_headers(query, data.column),
    }
}

fn is_filtered(query: &RunsQuery) -> bool {
    query.farm().is_some()
        || query.agent().is_some()
        || query.user().is_some()
        || query.status_value().is_some()
        || query.trigger_value().is_some()
}

fn status_href(query: &RunsQuery, status: RunStatus) -> (Option<String>, bool) {
    let active = query.status_value() == Some(status);
    let href = if active {
        query.url_without(&["status", "page"])
    } else {
        format!(
            "{}status={}",
            query.url_prefix(&["status", "page"]),
            status.as_str()
        )
    };
    (Some(href), active)
}

fn status_tile(
    query: &RunsQuery,
    status: RunStatus,
    (label, tone): (&'static str, &'static str),
    count: i64,
    note: Option<String>,
) -> RunKpiView {
    let (href, active) = status_href(query, status);
    RunKpiView {
        key: status.as_str(),
        label,
        value: count.to_string(),
        tone: Some(tone),
        href,
        active,
        note,
        hint: Some("Show only runs with this status; click again to clear"),
    }
}

const fn plain_tile(
    key: &'static str,
    label: &'static str,
    value: String,
    note: Option<String>,
) -> RunKpiView {
    RunKpiView {
        key,
        label,
        value,
        tone: None,
        href: None,
        active: false,
        note,
        hint: None,
    }
}

fn percent(part: i64, whole: i64) -> String {
    if whole <= 0 {
        return "no finished runs".to_owned();
    }
    format!(
        "{:.0}% of finished runs",
        (part as f64 / whole as f64) * 100.0
    )
}

fn duration(ms: Option<f64>) -> String {
    ms.map_or_else(|| "—".to_owned(), |v| format_duration_ms(v.round() as i64))
}

pub(super) fn kpis(query: &RunsQuery, stats: &RunStats) -> Vec<RunKpiView> {
    let finished = stats.succeeded + stats.failed + stats.timed_out + stats.cancelled;
    vec![
        plain_tile(
            "total",
            "Runs",
            stats.total.to_string(),
            Some("in the window".to_owned()),
        ),
        status_tile(
            query,
            RunStatus::Running,
            ("Running now", "accent"),
            stats.running,
            None,
        ),
        status_tile(
            query,
            RunStatus::Queued,
            ("Queued", "info"),
            stats.queued,
            None,
        ),
        status_tile(
            query,
            RunStatus::Succeeded,
            ("Succeeded", "ok"),
            stats.succeeded,
            Some(percent(stats.succeeded, finished)),
        ),
        status_tile(
            query,
            RunStatus::Failed,
            ("Failed", "err"),
            stats.failed,
            Some(format!(
                "{} timed out · {} cancelled",
                stats.timed_out, stats.cancelled
            )),
        ),
        plain_tile(
            "cost",
            "Cost",
            format_cost(stats.cost_microdollars),
            Some(format!(
                "{} tool calls · {} gateway requests",
                stats.tool_calls, stats.gateway_requests
            )),
        ),
        plain_tile(
            "duration",
            "p95 duration",
            duration(stats.p95_duration_ms),
            Some(format!("p50 {}", duration(stats.p50_duration_ms))),
        ),
    ]
}

pub(super) fn sort_headers(query: &RunsQuery, column: RunSortColumn) -> RunSortHeaders {
    let prefix = query.url_prefix(&["sort", "dir", "page"]);
    let started_desc = query.dir.as_deref() != Some("asc");
    let header = |col: RunSortColumn, label: &'static str, hint: &'static str| {
        let active = col == column;
        let descending = col != RunSortColumn::Started || started_desc;
        let next = if col == RunSortColumn::Started {
            SortHeaderView::next_dir(active, descending)
        } else {
            "desc"
        };
        SortHeaderView::new(
            (label, "sp-table__cell", hint),
            format!("{prefix}sort={}&dir={next}", col.key()),
            active,
            descending,
        )
    };
    RunSortHeaders {
        started: header(
            RunSortColumn::Started,
            "Started",
            "When the run started, or was queued",
        ),
        duration: header(RunSortColumn::Duration, "Duration", "Longest first"),
        tool_calls: header(
            RunSortColumn::ToolCalls,
            "Tool calls",
            "Most tool calls first",
        ),
        cost: header(RunSortColumn::Cost, "Cost", "Most expensive first"),
    }
}
