//! URL assembly for the governance page: tabs, sort headers, pagination.
//!
//! Every control on the page is a link, so a view is bookmarkable and the
//! server renders only what the URL asked for. That makes URL construction the
//! page's single most repeated operation, and it lives here so a control can
//! never drop a parameter another control set.

use serde::Serialize;

use super::{BASE_URL, GovernanceQuery};
use crate::repositories::governance::decision_log::DecisionSort;
use systemprompt_web_admin_core::ssr::list_view::{PageWindow, Pagination, paginate};

type Param = (&'static str, String);

fn params(query: &GovernanceQuery) -> Vec<Param> {
    let mut out: Vec<Param> = Vec::new();
    let mut push = |name: &'static str, value: Option<&String>| {
        if let Some(v) = value.filter(|v| !v.is_empty()) {
            out.push((name, v.clone()));
        }
    };
    push("tab", query.tab.as_ref());
    push("preset", query.preset.as_ref());
    push("from", query.from.as_ref());
    push("to", query.to.as_ref());
    push("group", query.group.as_ref());
    push("project", query.project.as_ref());
    push("policy", query.policy.as_ref());
    push("decision", query.decision.as_ref());
    push("category", query.category.as_ref());
    push("blocked", query.blocked.as_ref());
    push("q", query.q.as_ref());
    push("attention", query.attention.as_ref());
    push("sort", query.sort.as_ref());
    push("dir", query.dir.as_ref());
    if let Some(page) = query.page.filter(|p| *p > 0) {
        out.push(("page", page.to_string()));
    }
    out
}

pub(super) fn url_with(query: &GovernanceQuery, overrides: &[(&'static str, &str)]) -> String {
    let mut kept: Vec<Param> = params(query)
        .into_iter()
        .filter(|(name, _)| !overrides.iter().any(|(o, _)| o == name))
        .collect();
    for (name, value) in overrides {
        if !value.is_empty() {
            kept.push((name, (*value).to_owned()));
        }
    }
    render(&kept)
}

pub(super) fn filter_url(query: &GovernanceQuery, overrides: &[(&'static str, &str)]) -> String {
    let mut all: Vec<(&'static str, &str)> = overrides.to_vec();
    all.push(("page", ""));
    url_with(query, &all)
}

fn render(params: &[Param]) -> String {
    if params.is_empty() {
        return BASE_URL.to_owned();
    }
    let query = params
        .iter()
        .map(|(name, value)| format!("{name}={}", urlencoding::encode(value)))
        .collect::<Vec<_>>()
        .join("&");
    format!("{BASE_URL}?{query}")
}

#[derive(Debug, Serialize)]
pub(super) struct ColumnHeader {
    pub(super) label: &'static str,
    pub(super) class: &'static str,
    pub(super) url: Option<String>,
    pub(super) active: bool,
    pub(super) aria_sort: &'static str,
    pub(super) indicator: &'static str,
    pub(super) hint: Option<&'static str>,
}

impl ColumnHeader {
    pub(super) const fn plain(label: &'static str, class: &'static str) -> Self {
        Self {
            label,
            class,
            url: None,
            active: false,
            aria_sort: "none",
            indicator: "",
            hint: None,
        }
    }

    pub(super) fn sortable(
        label: &'static str,
        class: &'static str,
        key: &'static str,
        query: &GovernanceQuery,
        sort: DecisionSort,
    ) -> Self {
        let ascending = sort.ascending;
        let active = sort.key == key;
        let next = if active && ascending { "desc" } else { "asc" };
        Self {
            label,
            class,
            url: Some(filter_url(query, &[("sort", key), ("dir", next)])),
            active,
            aria_sort: if !active {
                "none"
            } else if ascending {
                "ascending"
            } else {
                "descending"
            },
            indicator: if !active {
                ""
            } else if ascending {
                "\u{2191}"
            } else {
                "\u{2193}"
            },
            hint: None,
        }
    }
}

pub(super) fn build_pagination(
    query: &GovernanceQuery,
    window: PageWindow,
    noun: &'static str,
) -> Pagination {
    Pagination {
        noun,
        ..paginate(window, |page| {
            url_with(query, &[("page", &page.to_string())])
        })
    }
}
