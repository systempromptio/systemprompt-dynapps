//! Summary strip + sortable headers for the Trace Explorer list page.
//!
//! Split from `view.rs`, which keeps the filter ribbon, chips and pagination.
//! Both halves share `preserved_query_string` so every link on the page carries
//! the same filter and time-range state.

use crate::handlers::ssr::format::{format_cost, format_duration_ms, format_token_total};
use crate::repositories::traces::TraceStats;

use super::context::{TraceStatsView, TracesSortHeaders};
use super::view::preserved_query_string;
use super::{BASE_URL, TraceListQuery};
use crate::handlers::ssr::types::SortHeaderView;

pub(super) fn serde_stats(query: &TraceListQuery, s: &TraceStats) -> TraceStatsView {
    TraceStatsView {
        total_traces: s.total_traces,
        error_count: s.error_count,
        deny_count: s.deny_count,
        deny_url: toggle_flag_url(query, "deny_only"),
        error_url: toggle_flag_url(query, "error_only"),
        deny_active: query.deny_only.as_deref() == Some("true"),
        error_active: query.error_only.as_deref() == Some("true"),
        lifecycle_count: s.lifecycle_count,
        lifecycle_url: toggle_flag_url(query, "include_lifecycle"),
        lifecycle_active: query.include_lifecycle.as_deref() == Some("true"),
        traces_note: traces_note(query, s),
        active_note: format!("across {} traces with requests", s.active_traces),
        cost_display: format_cost(s.total_cost_microdollars),
        tokens_display: format_token_total(s.total_tokens),
        p50_display: format_duration_ms(s.p50_active_ms),
        p95_display: format_duration_ms(s.p95_active_ms),
        p99_display: format_duration_ms(s.p99_active_ms),
    }
}

fn traces_note(query: &TraceListQuery, s: &TraceStats) -> String {
    if query.include_lifecycle.as_deref() == Some("true") {
        return "in window · lifecycle-only shown".to_owned();
    }
    if s.lifecycle_count == 0 {
        return "in window".to_owned();
    }
    format!("in window · {} lifecycle-only hidden", s.lifecycle_count)
}

fn toggle_flag_url(query: &TraceListQuery, flag: &str) -> String {
    let already_on = match flag {
        "deny_only" => query.deny_only.as_deref() == Some("true"),
        "include_lifecycle" => query.include_lifecycle.as_deref() == Some("true"),
        _ => query.error_only.as_deref() == Some("true"),
    };
    let qs = preserved_query_string(query, &[flag, "page"]);
    if already_on {
        return if qs.is_empty() {
            BASE_URL.to_owned()
        } else {
            format!("{BASE_URL}?{qs}")
        };
    }
    if qs.is_empty() {
        format!("{BASE_URL}?{flag}=true")
    } else {
        format!("{BASE_URL}?{qs}&{flag}=true")
    }
}

pub(super) fn build_sort_headers(
    query: &TraceListQuery,
    active_col: &str,
    active_dir: &str,
) -> TracesSortHeaders {
    let qs = preserved_query_string(query, &["sort", "dir", "page"]);
    let prefix = if qs.is_empty() {
        format!("{BASE_URL}?")
    } else {
        format!("{BASE_URL}?{qs}&")
    };
    let header = |key: &str, label: &'static str, class: &'static str, hint: &'static str| {
        let active = key == active_col;
        let descending = active_dir != "asc";
        let next_dir = SortHeaderView::next_dir(active, descending);
        SortHeaderView::new(
            (label, class, hint),
            format!("{prefix}sort={key}&dir={next_dir}"),
            active,
            descending,
        )
    };
    TracesSortHeaders {
        started: header(
            "started_at",
            "Started",
            "sp-col-started",
            "First event on the trace, in local time",
        ),
        activity: header(
            "spans",
            "Activity",
            "sp-col-spans",
            "Gateway requests, then governance decisions and tool calls",
        ),
        tokens: header(
            "tokens",
            "Tokens",
            "sp-col-tokens",
            "Total tokens, split input / output",
        ),
        cost: header(
            "cost",
            "Cost (measured)",
            "sp-col-cost",
            "Billed cost across the trace, where the route carries a price",
        ),
        duration: header(
            "duration",
            "Duration",
            "sp-col-duration",
            "Summed request latency, over the first-to-last event window",
        ),
    }
}
