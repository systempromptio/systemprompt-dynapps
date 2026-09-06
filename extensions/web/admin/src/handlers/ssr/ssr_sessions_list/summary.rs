//! The KPI strip and the sortable headers for the sessions list.
//!
//! Split from `view`, which keeps the filter ribbon, the scope form and the
//! pagination. Both halves build their links from the same
//! `preserved_query_string`, so a sort and a page turn carry identical state.

use crate::handlers::ssr::format::{format_cost, format_token_total};
use crate::handlers::ssr::types::SortHeaderView;
use crate::repositories::analytics::sessions_list::{
    SessionListKpis, SessionSortColumn, SessionSortDir,
};

use super::context::{SessionsSortHeaders, StatsView};
use super::view::preserved_query_string;
use super::{BASE_URL, SessionListQuery};

pub(super) fn build_sort_headers(
    query: &SessionListQuery,
    active_col: SessionSortColumn,
    active_dir: SessionSortDir,
) -> SessionsSortHeaders {
    let qs = preserved_query_string(query, &["sort", "dir", "page"]);
    let prefix = if qs.is_empty() {
        format!("{BASE_URL}?")
    } else {
        format!("{BASE_URL}?{qs}&")
    };
    let is_desc = active_dir == SessionSortDir::Desc;
    let header = |col: SessionSortColumn, label: &'static str, class: &'static str, hint| {
        let active = col == active_col;
        let next_dir = if active && is_desc { "asc" } else { "desc" };
        SortHeaderView {
            label,
            class,
            hint,
            url: format!("{prefix}sort={}&dir={next_dir}", col.as_str()),
            active,
            aria_sort: if active {
                if is_desc { "descending" } else { "ascending" }
            } else {
                "none"
            },
            indicator: if active {
                if is_desc { "▼" } else { "▲" }
            } else {
                "↕"
            },
        }
    };
    SessionsSortHeaders {
        started: header(
            SessionSortColumn::StartedAt,
            "Started",
            "sp-col-date",
            "First recorded activity on the session, in local time",
        ),
        duration: header(
            SessionSortColumn::Duration,
            "Duration",
            "sp-col-duration",
            "First to last recorded activity",
        ),
        requests: header(
            SessionSortColumn::Requests,
            "Reqs",
            "sp-table__cell--num",
            "Gateway requests issued in this session",
        ),
        tokens: header(
            SessionSortColumn::Tokens,
            "Tokens",
            "sp-table__cell--num",
            "Input plus output tokens across the session",
        ),
        cost: header(
            SessionSortColumn::Cost,
            "Cost",
            "sp-table__cell--num",
            "Billed cost across the session",
        ),
    }
}

pub(super) fn stats_view(k: &SessionListKpis) -> StatsView {
    let (sessions_delta, sessions_delta_dir) =
        delta(k.total_sessions, k.prev_total_sessions, Rising::Neutral);
    let (requests_delta, requests_delta_dir) =
        delta(k.total_requests, k.prev_total_requests, Rising::Neutral);
    let (tokens_delta, tokens_delta_dir) = delta(k.total_tokens, k.prev_total_tokens, Rising::Bad);
    let (cost_delta, cost_delta_dir) = delta(
        k.total_cost_microdollars,
        k.prev_total_cost_microdollars,
        Rising::Bad,
    );
    StatsView {
        total_sessions: k.total_sessions,
        error_sessions: k.error_sessions,
        total_requests: k.total_requests,
        total_tool_uses: k.total_tool_uses,
        tokens_display: format_token_total(k.total_tokens),
        cost_display: format_cost(k.total_cost_microdollars),
        sessions_delta,
        sessions_delta_dir,
        requests_delta,
        requests_delta_dir,
        tokens_delta,
        tokens_delta_dir,
        cost_delta,
        cost_delta_dir,
    }
}

// Why: more sessions is neither good nor bad, but more spend is. The design
// system carries both readings, and picking the wrong one paints a rising bill
// in the colour of a healthy trend.
#[derive(Clone, Copy)]
enum Rising {
    Neutral,
    Bad,
}

// Why: a previous window of zero has no percentage to report, so the KPI shows
// no delta at all rather than an infinite rise. An unchanged figure shows the
// number without a direction, because there is no direction to show.
fn delta(current: i64, previous: i64, rising: Rising) -> (Option<String>, Option<&'static str>) {
    if previous <= 0 {
        return (None, None);
    }
    let pct = ((current - previous) as f64 / previous as f64) * 100.0;
    let rounded = pct.round();
    if rounded == 0.0 {
        return (Some("0%".to_owned()), None);
    }
    let dir = match (rounded > 0.0, rising) {
        (true, Rising::Neutral) => "up",
        (true, Rising::Bad) => "up-bad",
        (false, Rising::Neutral) => "down",
        (false, Rising::Bad) => "down-good",
    };
    (Some(format!("{rounded:+.0}%")), Some(dir))
}
