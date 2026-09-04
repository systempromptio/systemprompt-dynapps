//! The pieces every filtered entity-list page renders the same way.
//!
//! Every filtered list page binds to the same URL contract —
//! `?preset=&from=&to=&<facet>=&page=` — and feeds the same
//! `components/time-range`, `components/identity-filter-ribbon` and pagination
//! partials. The types below are the serialisation those partials read, and
//! the functions build them from a page's own query parameters expressed as a
//! `(name, value)` slice, so a list page supplies its parameter list and
//! inherits the behaviour rather than copying it.

pub(crate) use systemprompt_web_shared::pagination::PageWindow;

use serde::Serialize;

use crate::types::{Project, UserContext};
use crate::util::project_scope::ProjectScope;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SelectOptionView {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HiddenFieldView {
    pub name: String,
    pub value: String,
}

// Why: The `components/project-filter` partial's context: the project select an
// admin narrows a listing with. Hidden for everyone else — a non-admin's
// scope is their own project and nothing on the page can widen it.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProjectFilterView {
    pub show: bool,
    pub base_url: String,
    pub options: Vec<SelectOptionView>,
    pub hidden: Vec<HiddenFieldView>,
}

pub(crate) fn project_filter_view(
    user_ctx: &UserContext,
    scope: ProjectScope,
    base_url: &str,
    hidden: Vec<(String, String)>,
) -> ProjectFilterView {
    let selected = scope.selected();
    let mut options = vec![SelectOptionView {
        value: String::new(),
        label: "All projects".to_owned(),
        selected: selected.is_none(),
    }];
    options.extend(Project::ALL.into_iter().map(|p| SelectOptionView {
        value: p.as_str().to_owned(),
        label: p.label().to_owned(),
        selected: selected == Some(p),
    }));
    ProjectFilterView {
        show: user_ctx.is_console,
        base_url: base_url.to_owned(),
        options,
        hidden: hidden
            .into_iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(name, value)| HiddenFieldView { name, value })
            .collect(),
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct TimeRangeContext {
    pub(crate) preset: String,
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) base_url: &'static str,
    pub(crate) query: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct Preserved {
    pub(crate) name: &'static str,
    pub(crate) value: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct AnnotatedOption {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) count: i64,
    pub(crate) selected: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct Chip {
    pub(crate) group_label: &'static str,
    pub(crate) label: String,
    pub(crate) value: String,
    pub(crate) remove_url: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct Pagination {
    pub(crate) current_page: i64,
    pub(crate) total_pages: i64,
    // Why: 1-based row range for "Showing 1-50 of 54"; `first_row` is 0 only
    // when the page is empty.
    pub(crate) first_row: i64,
    pub(crate) last_row: i64,
    pub(crate) total_rows: i64,
    pub(crate) noun: &'static str,
    pub(crate) has_prev: bool,
    pub(crate) has_next: bool,
    pub(crate) prev_url: Option<String>,
    pub(crate) next_url: Option<String>,
}
