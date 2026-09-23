//! View-model assembly for the sessions list: preserved query state, chips,
//! sortable headers, pagination, the scope form, and the KPI strip.
//!
//! Every link the page emits is built from `preserved_query_string`, so the
//! window, the scope and the filters survive a sort, a page turn, or a chip
//! removal.

use systemprompt::identifiers::UserId;

use crate::handlers::ssr::list_view::{
    AnnotatedOption, Chip, PageWindow, Pagination, Preserved, ScopeFilterView, TimeRangeContext,
    scope_filter_view,
};
use crate::repositories::governance::filter_options::FilterOption;
use crate::repositories::scope::ScopeRequest;
use crate::types::UserContext;
use crate::util::time_range::TimeRange;

use super::context::SessionFilterOptionsView;
use super::{BASE_URL, SessionListQuery};
use crate::handlers::ssr::list_view::{paginate, query_string_dropping};

pub(super) fn preserved_query_string(query: &SessionListQuery, drop: &[&str]) -> String {
    let pairs: [(&str, Option<&str>); 10] = [
        ("group", query.group.as_deref()),
        ("project", query.project.as_deref()),
        ("preset", query.preset.as_deref()),
        ("from", query.from.as_deref()),
        ("to", query.to.as_deref()),
        ("user_id", query.user_id.as_ref().map(UserId::as_str)),
        ("error_only", query.error_only.as_deref()),
        ("side", query.side.as_deref()),
        ("sort", query.sort.as_deref()),
        ("dir", query.dir.as_deref()),
    ];
    query_string_dropping(&pairs, &[], drop)
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
        rejected: range.rejected_bounds,
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
    if query.side.as_deref() == Some("1") {
        out.push(Preserved {
            name: "side",
            value: "1".to_owned(),
        });
    }
    out
}

pub(super) struct SessionScopeFilterArgs<'a> {
    pub request: &'a ScopeRequest,
    pub query: &'a SessionListQuery,
    pub range: TimeRange,
    pub preset: &'a str,
}

pub(super) async fn scope_filter(
    pool: &sqlx::PgPool,
    user_ctx: &UserContext,
    args: &SessionScopeFilterArgs<'_>,
) -> ScopeFilterView {
    let SessionScopeFilterArgs {
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

// Why: The errors-only card is a toggle: clicking it narrows to the sessions it
// counts, clicking it again clears the flag.
pub(super) fn error_toggle_url(query: &SessionListQuery, active: bool) -> String {
    if active {
        url_with(query, &["error_only", "page"], "")
    } else {
        url_with(query, &["error_only", "page"], "error_only=true")
    }
}

// Why: side calls are hidden by default because a probe-only conversation is
// noise to the person asking "what did people talk about"; the toggle shows
// them for the person asking "what did we pay for".
pub(super) fn side_toggle_url(query: &SessionListQuery, active: bool) -> String {
    if active {
        url_with(query, &["side", "page"], "")
    } else {
        url_with(query, &["side", "page"], "side=1")
    }
}

pub(super) fn build_pagination(query: &SessionListQuery, window: PageWindow) -> Pagination {
    paginate(window, |page| {
        url_with(query, &["page"], &format!("page={page}"))
    })
}
