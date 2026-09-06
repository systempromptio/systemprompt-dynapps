//! Link building for the contexts list: the query state every sort link, page
//! link and chip carries, plus the sortable headers and the pagination footer.

use systemprompt::identifiers::UserId;
use urlencoding::encode as urlencode;

use crate::handlers::ssr::list_view::{PageWindow, Pagination};
use crate::repositories::analytics::contexts_list::{ContextSortColumn, ContextSortDir};

use crate::handlers::ssr::types::{SortHeaderView, TabLinkView};

use super::context::ContextsSortHeaders;
use super::{BASE_URL, ContextsListQuery, PAGE_SIZE};

pub(super) fn preserved_query_string(query: &ContextsListQuery, drop: &[&str]) -> String {
    let pairs: [(&str, Option<&str>); 10] = [
        ("group", query.group.as_deref()),
        ("project", query.project.as_deref()),
        ("user_id", query.user_id.as_ref().map(UserId::as_str)),
        ("model", query.model.as_deref()),
        ("q", query.q.as_deref()),
        ("since", query.since.as_deref()),
        ("preset", query.preset.as_deref()),
        ("view", query.view.as_deref()),
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

pub(super) fn build_pagination(query: &ContextsListQuery, window: PageWindow) -> Pagination {
    let page = window.index;
    let qs = preserved_query_string(query, &["page"]);
    let prefix = if qs.is_empty() {
        format!("{BASE_URL}?")
    } else {
        format!("{BASE_URL}?{qs}&")
    };
    let prev_url = (page > 0).then(|| format!("{prefix}page={}", page.saturating_sub(1)));
    let next_url = (page + 1 < window.total_pages).then(|| format!("{prefix}page={}", page + 1));
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
    query: &ContextsListQuery,
    active_col: ContextSortColumn,
    active_dir: ContextSortDir,
) -> ContextsSortHeaders {
    let qs = preserved_query_string(query, &["sort", "dir", "page"]);
    let prefix = if qs.is_empty() {
        format!("{BASE_URL}?")
    } else {
        format!("{BASE_URL}?{qs}&")
    };
    let is_desc = active_dir == ContextSortDir::Desc;
    let header = |col: ContextSortColumn, label: &'static str, class: &'static str, hint| {
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
    ContextsSortHeaders {
        activity: header(
            ContextSortColumn::Activity,
            "Last activity",
            "sp-col-date",
            "Most recent request on the context, in local time",
        ),
        requests: header(
            ContextSortColumn::Requests,
            "Reqs",
            "sp-table__cell--num",
            "Gateway requests issued against this context",
        ),
        messages: header(
            ContextSortColumn::Messages,
            "Msgs",
            "sp-table__cell--num",
            "Messages stored in the transcript",
        ),
        tokens: header(
            ContextSortColumn::Tokens,
            "Tokens",
            "sp-table__cell--num",
            "Input plus output tokens across the context",
        ),
        cost: header(
            ContextSortColumn::Cost,
            "Cost",
            "sp-table__cell--num",
            "Billed cost across the context",
        ),
    }
}

// Why: The page index the caller asked for, clamped to zero, and the offset the
// query binds for it.
pub(super) fn page_offset(page: i64) -> i64 {
    page.max(0) * PAGE_SIZE
}

// Why: The two views of the same rows: one context per row, or one person per
// row with their contexts nested. Links, not buttons, so a view is bookmarkable
// and survives a reload without JavaScript.
pub(super) fn view_tabs(query: &ContextsListQuery, active: &str) -> Vec<TabLinkView> {
    let qs = preserved_query_string(query, &["view", "page"]);
    let prefix = if qs.is_empty() {
        format!("{BASE_URL}?")
    } else {
        format!("{BASE_URL}?{qs}&")
    };
    [("contexts", "Contexts"), ("users", "By user")]
        .into_iter()
        .map(|(slug, label)| TabLinkView {
            slug,
            label,
            href: format!("{prefix}view={slug}"),
            is_active: slug == active,
            count: None,
        })
        .collect()
}
