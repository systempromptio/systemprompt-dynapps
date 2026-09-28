//! The pieces every filtered entity-list page renders the same way.
//!
//! Every filtered list page binds to the same URL contract —
//! `?preset=&from=&to=&<facet>=&page=` — and feeds the same
//! `components/time-range`, `components/identity-filter-ribbon` and pagination
//! partials. The types below are the serialisation those partials read, and
//! the functions build them from a page's own query parameters expressed as a
//! `(name, value)` slice, so a list page supplies its parameter list and
//! inherits the behaviour rather than copying it.
//!
//! The `components/scope-filter` selects (`group`, `project`) are shown to
//! console admins only, and their option lists are read inside
//! [`scope_filter_view`] so every list page inherits the same filter from one
//! call. A scoped caller is offered only the containers their participation
//! names, and a failed read degrades to the "all" option rather than failing
//! the page.

pub use systemprompt_web_shared::pagination::PageWindow;

pub const DEFAULT_PAGE_SIZE: i64 = 50;

use serde::Serialize;
use std::collections::BTreeMap;

use sqlx::PgPool;

use crate::repositories::scope::ScopeRequest;
use crate::repositories::scope::containers::{list_group_options, list_project_options};
use crate::types::UserContext;

#[derive(Debug, Clone, Serialize)]
pub struct SelectOptionView {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct HiddenFieldView {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ScopeFilterView {
    pub show: bool,
    pub base_url: String,
    pub groups: Vec<SelectOptionView>,
    pub projects: Vec<SelectOptionView>,
    pub hidden: Vec<HiddenFieldView>,
}

pub async fn scope_filter_view(
    pool: &PgPool,
    user_ctx: &UserContext,
    scope: &ScopeRequest,
    base_url: &str,
    hidden: Vec<(String, String)>,
) -> ScopeFilterView {
    if !user_ctx.has_scoped_console() {
        return ScopeFilterView::default();
    }
    let (group_rows, project_rows) =
        tokio::join!(list_group_options(pool), list_project_options(pool));
    let may_see_container =
        |ids: &[String], id: &str| user_ctx.is_console || ids.iter().any(|v| v == id);
    let groups = group_rows
        .unwrap_or_default()
        .into_iter()
        .filter(|g| may_see_container(&user_ctx.participation.groups, g.id.as_str()))
        .map(|g| (g.id.as_str().to_owned(), g.name))
        .collect();
    let projects = project_rows
        .unwrap_or_default()
        .into_iter()
        .filter(|p| may_see_container(&user_ctx.participation.projects, p.id.as_str()))
        .map(|p| (p.id.as_str().to_owned(), p.name))
        .collect();
    scope_filter_from_names(
        user_ctx,
        scope,
        base_url,
        hidden,
        ScopeNames {
            groups: &groups,
            projects: &projects,
        },
    )
}

#[derive(Debug, Clone, Copy)]
pub struct ScopeNames<'a> {
    pub groups: &'a BTreeMap<String, String>,
    pub projects: &'a BTreeMap<String, String>,
}

pub fn scope_filter_from_names(
    user_ctx: &UserContext,
    scope: &ScopeRequest,
    base_url: &str,
    hidden: Vec<(String, String)>,
    names: ScopeNames<'_>,
) -> ScopeFilterView {
    let ScopeNames { groups, projects } = names;
    ScopeFilterView {
        show: user_ctx.has_scoped_console(),
        base_url: base_url.to_owned(),
        groups: options(
            "All groups",
            scope.group.as_deref(),
            groups.iter().map(|(id, name)| (id.clone(), name.clone())),
        ),
        projects: options(
            "All projects",
            scope.project.as_deref(),
            projects.iter().map(|(id, name)| (id.clone(), name.clone())),
        ),
        hidden: hidden
            .into_iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(name, value)| HiddenFieldView { name, value })
            .collect(),
    }
}

fn options(
    all_label: &str,
    selected: Option<&str>,
    rows: impl Iterator<Item = (String, String)>,
) -> Vec<SelectOptionView> {
    let mut out = vec![SelectOptionView {
        value: String::new(),
        label: all_label.to_owned(),
        selected: selected.is_none(),
    }];
    out.extend(rows.map(|(value, label)| SelectOptionView {
        selected: selected == Some(value.as_str()),
        value,
        label,
    }));
    out
}

#[derive(Debug, Serialize)]
pub struct TimeRangeContext {
    pub preset: String,
    pub from: String,
    pub to: String,
    pub base_url: &'static str,
    pub query: &'static str,
    pub rejected: bool,
}

#[derive(Debug, Serialize)]
pub struct Preserved {
    pub name: &'static str,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct AnnotatedOption {
    pub id: String,
    pub label: String,
    pub count: i64,
    pub selected: bool,
}

#[derive(Debug, Serialize)]
pub struct Chip {
    pub group_label: &'static str,
    pub label: String,
    pub value: String,
    pub remove_url: String,
}

pub fn query_string_dropping(
    pairs: &[(&str, Option<&str>)],
    extra: &[(&str, Option<i64>)],
    drop: &[&str],
) -> String {
    let mut parts: Vec<String> = pairs
        .iter()
        .filter(|(name, _)| !drop.contains(name))
        .filter_map(|(name, value)| {
            value
                .filter(|v| !v.is_empty())
                .map(|v| format!("{name}={}", urlencoding::encode(v)))
        })
        .collect();
    parts.extend(
        extra
            .iter()
            .filter(|(name, _)| !drop.contains(name))
            .filter_map(|(name, value)| value.map(|v| format!("{name}={v}"))),
    );
    parts.join("&")
}

pub fn paginate(window: PageWindow, page_url: impl Fn(i64) -> String) -> Pagination {
    let page = window.index;
    let prev_url = (page > 0).then(|| page_url(page - 1));
    let next_url = (page + 1 < window.total_pages).then(|| page_url(page + 1));
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

pub fn paginate_prefixed(window: PageWindow, prefix: &str) -> Pagination {
    paginate(window, |page| format!("{prefix}page={page}"))
}

#[derive(Debug, Serialize)]
pub struct Pagination {
    pub current_page: i64,
    pub total_pages: i64,
    pub first_row: i64,
    pub last_row: i64,
    pub total_rows: i64,
    pub noun: &'static str,
    pub has_prev: bool,
    pub has_next: bool,
    pub prev_url: Option<String>,
    pub next_url: Option<String>,
}
