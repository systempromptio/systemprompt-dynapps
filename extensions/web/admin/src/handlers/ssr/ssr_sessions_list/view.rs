//! View-model assembly for the sessions list: preserved query state, chips,
//! sortable headers, pagination, the scope form, and the KPI strip.
//!
//! Every link the page emits is built from `preserved_query_string`, so the
//! window, the scope and the filters survive a sort, a page turn, or a chip
//! removal.

use systemprompt::identifiers::UserId;
use urlencoding::encode as urlencode;

use crate::handlers::ssr::format::{format_cost, format_token_total};
use crate::handlers::ssr::list_view::{
    AnnotatedOption, Chip, PageWindow, Pagination, Preserved, ScopeFilterView, TimeRangeContext,
    scope_filter_view,
};
use crate::repositories::analytics::sessions_list::{
    SessionListKpis, SessionSortColumn, SessionSortDir,
};
use crate::repositories::governance::filter_options::FilterOption;
use crate::repositories::scope::ScopeRequest;
use crate::types::UserContext;
use crate::util::time_range::TimeRange;

use crate::handlers::ssr::types::SortHeaderView;

use super::context::{SessionFilterOptionsView, SessionsSortHeaders, StatsView};
use super::{BASE_URL, SessionListQuery};

pub(super) fn preserved_query_string(query: &SessionListQuery, drop: &[&str]) -> String {
    let pairs: [(&str, Option<&str>); 9] = [
        ("group", query.group.as_deref()),
        ("project", query.project.as_deref()),
        ("preset", query.preset.as_deref()),
        ("from", query.from.as_deref()),
        ("to", query.to.as_deref()),
        ("user_id", query.user_id.as_ref().map(UserId::as_str)),
        ("error_only", query.error_only.as_deref()),
        ("sort", query.sort.as_deref()),
        ("dir", query.dir.as_deref()),
    ];
    pairs
        .iter()
        .filter(|(name, _)| !drop.contains(name))
        .filter_map(|(name, val)| {
            val.filter(|s| !s.is_empty())
                .map(|v| format!("{}={}", name, urlencode(v)))
        })
        .collect::<Vec<_>>()
        .join("&")
}

fn url_with(query: &SessionListQuery, drop: &[&str], extra: &str) -> String {
    let qs = preserved_query_string(query, drop);
    match (qs.is_empty(), extra.is_empty()) {
        (true, true) => BASE_URL.to_owned(),
        (true, false) => format!("{BASE_URL}?{extra}"),
        (false, true) => format!("{BASE_URL}?{qs}"),
        (false, false) => format!("{BASE_URL}?{qs}&{extra}"),
    }
}

pub(super) fn time_range_context(range: TimeRange, preset: &str) -> TimeRangeContext {
    TimeRangeContext {
        preset: preset.to_owned(),
        from: range.from.to_rfc3339(),
        to: range.to.to_rfc3339(),
        base_url: BASE_URL,
        query: "",
    }
}

pub(super) fn build_preserved(
    query: &SessionListQuery,
    range: TimeRange,
    preset: &str,
) -> Vec<Preserved> {
    let mut out = vec![
        Preserved {
            name: "preset",
            value: preset.to_owned(),
        },
        Preserved {
            name: "from",
            value: range.from.to_rfc3339(),
        },
        Preserved {
            name: "to",
            value: range.to.to_rfc3339(),
        },
    ];
    if query.error_only.as_deref() == Some("true") {
        out.push(Preserved {
            name: "error_only",
            value: "true".to_owned(),
        });
    }
    out
}

pub(super) struct ScopeFilterArgs<'a> {
    pub request: &'a ScopeRequest,
    pub query: &'a SessionListQuery,
    pub range: TimeRange,
    pub preset: &'a str,
}

pub(super) async fn scope_filter(
    pool: &sqlx::PgPool,
    user_ctx: &UserContext,
    args: &ScopeFilterArgs<'_>,
) -> ScopeFilterView {
    let ScopeFilterArgs {
        request,
        query,
        range,
        preset,
    } = *args;
    let mut hidden: Vec<(String, String)> = build_preserved(query, range, preset)
        .into_iter()
        .map(|p| (p.name.to_owned(), p.value))
        .collect();
    for (name, value) in [
        ("user_id", query.user_id.as_ref().map(UserId::as_str)),
        ("sort", query.sort.as_deref()),
        ("dir", query.dir.as_deref()),
    ] {
        hidden.push((name.to_owned(), value.unwrap_or_default().to_owned()));
    }
    scope_filter_view(pool, user_ctx, request, BASE_URL, hidden).await
}

pub(super) fn build_chips(query: &SessionListQuery) -> Vec<Chip> {
    let Some(user) = query
        .user_id
        .as_ref()
        .map(UserId::as_str)
        .filter(|s| !s.is_empty())
    else {
        return Vec::new();
    };
    vec![Chip {
        group_label: "User",
        label: user.to_owned(),
        value: user.to_owned(),
        remove_url: url_with(query, &["user_id"], ""),
    }]
}

pub(super) fn annotate_options(
    users: &[FilterOption],
    selected: Option<&str>,
) -> SessionFilterOptionsView {
    SessionFilterOptionsView {
        users: users
            .iter()
            .map(|o| AnnotatedOption {
                id: o.id.clone(),
                label: o.label.clone(),
                count: o.count,
                selected: selected.is_some_and(|s| s == o.id),
            })
            .collect(),
    }
}

/// The errors-only card is a toggle: clicking it narrows to the sessions it
/// counts, clicking it again clears the flag.
pub(super) fn error_toggle_url(query: &SessionListQuery, active: bool) -> String {
    if active {
        url_with(query, &["error_only", "page"], "")
    } else {
        url_with(query, &["error_only", "page"], "error_only=true")
    }
}

pub(super) fn build_pagination(query: &SessionListQuery, window: PageWindow) -> Pagination {
    let page = window.index;
    let prev_url = (page > 0).then(|| {
        url_with(
            query,
            &["page"],
            &format!("page={}", page.saturating_sub(1)),
        )
    });
    let next_url = (page + 1 < window.total_pages)
        .then(|| url_with(query, &["page"], &format!("page={}", page + 1)));
    let (first_row, last_row) = window.bounds();
    Pagination {
        current_page: page + 1,
        total_pages: window.total_pages,
        first_row,
        last_row,
        total_rows: window.total_rows,
        noun: window.noun,
        has_prev: prev_url.is_some(),
        has_next: next_url.is_some(),
        prev_url,
        next_url,
    }
}

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
