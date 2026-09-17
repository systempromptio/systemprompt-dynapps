//! View-model assembly for the artifacts list: preserved query state, chips,
//! pagination, the scope form and the toggles.

use systemprompt::identifiers::{SessionId, UserId};
use urlencoding::encode as urlencode;

use crate::handlers::ssr::list_view::{
    AnnotatedOption, Chip, PageWindow, Pagination, Preserved, ScopeFilterView, TimeRangeContext,
    scope_filter_view,
};
use crate::repositories::governance::filter_options::FilterOption;
use crate::repositories::scope::ScopeRequest;
use crate::types::UserContext;
use crate::util::time_range::TimeRange;

use super::context::ArtifactFilterOptionsView;
use super::{ArtifactListQuery, BASE_URL};

fn query_pairs(query: &ArtifactListQuery) -> [(&'static str, Option<&str>); 12] {
    [
        ("group", query.group.as_deref()),
        ("project", query.project.as_deref()),
        ("preset", query.preset.as_deref()),
        ("from", query.from.as_deref()),
        ("to", query.to.as_deref()),
        ("user_id", query.user_id.as_ref().map(UserId::as_str)),
        ("session", query.session.as_ref().map(SessionId::as_str)),
        ("tool", query.tool.as_deref()),
        ("source", query.source.as_deref()),
        ("skill", query.skill.as_deref()),
        ("errors", query.errors.as_deref()),
        ("all", query.all.as_deref()),
    ]
}

pub(super) fn preserved_query_string(query: &ArtifactListQuery, drop: &[&str]) -> String {
    query_pairs(query)
        .iter()
        .filter(|(name, _)| !drop.contains(name))
        .filter_map(|(name, val)| {
            val.filter(|s| !s.is_empty())
                .map(|v| format!("{}={}", name, urlencode(v)))
        })
        .collect::<Vec<_>>()
        .join("&")
}

fn url_with(query: &ArtifactListQuery, drop: &[&str], extra: &str) -> String {
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
    query: &ArtifactListQuery,
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
    for (name, value) in [
        ("session", query.session.as_ref().map(SessionId::as_str)),
        ("tool", query.tool.as_deref()),
        ("source", query.source.as_deref()),
        ("skill", query.skill.as_deref()),
        ("errors", query.errors.as_deref()),
        ("all", query.all.as_deref()),
    ] {
        if let Some(value) = value.filter(|v| !v.is_empty()) {
            out.push(Preserved {
                name,
                value: value.to_owned(),
            });
        }
    }
    out
}

pub(super) struct ArtifactScopeFilterArgs<'a> {
    pub request: &'a ScopeRequest,
    pub query: &'a ArtifactListQuery,
    pub range: TimeRange,
    pub preset: &'a str,
}

pub(super) async fn scope_filter(
    pool: &sqlx::PgPool,
    user_ctx: &UserContext,
    args: &ArtifactScopeFilterArgs<'_>,
) -> ScopeFilterView {
    let ArtifactScopeFilterArgs {
        request,
        query,
        range,
        preset,
    } = *args;
    let mut hidden: Vec<(String, String)> = build_preserved(query, range, preset)
        .into_iter()
        .map(|p| (p.name.to_owned(), p.value))
        .collect();
    hidden.push((
        "user_id".to_owned(),
        query
            .user_id
            .as_ref()
            .map(UserId::as_str)
            .unwrap_or_default()
            .to_owned(),
    ));
    scope_filter_view(pool, user_ctx, request, BASE_URL, hidden).await
}

pub(super) fn build_chips(query: &ArtifactListQuery) -> Vec<Chip> {
    let mut chips = Vec::new();
    let facets: [(&'static str, &'static str, Option<&str>); 5] = [
        (
            "User",
            "user_id",
            query.user_id.as_ref().map(UserId::as_str),
        ),
        (
            "Session",
            "session",
            query.session.as_ref().map(SessionId::as_str),
        ),
        ("Tool", "tool", query.tool.as_deref()),
        ("Source", "source", query.source.as_deref()),
        ("Skill", "skill", query.skill.as_deref()),
    ];
    for (group_label, name, value) in facets {
        if let Some(value) = value.filter(|v| !v.is_empty()) {
            chips.push(Chip {
                group_label,
                label: value.to_owned(),
                value: value.to_owned(),
                remove_url: url_with(query, &[name, "page"], ""),
            });
        }
    }
    chips
}

pub(super) fn annotate_options(
    users: &[FilterOption],
    selected: Option<&str>,
) -> ArtifactFilterOptionsView {
    ArtifactFilterOptionsView {
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

// Why: structured results are the default lens; the toggle widens to every
// result and back without losing the rest of the query.
pub(super) fn all_toggle_url(query: &ArtifactListQuery, showing_all: bool) -> String {
    if showing_all {
        url_with(query, &["all", "page"], "")
    } else {
        url_with(query, &["all", "page"], "all=1")
    }
}

pub(super) fn errors_toggle_url(query: &ArtifactListQuery, active: bool) -> String {
    if active {
        url_with(query, &["errors", "page"], "")
    } else {
        url_with(query, &["errors", "page"], "errors=1")
    }
}

pub(super) fn build_pagination(query: &ArtifactListQuery, window: PageWindow) -> Pagination {
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
