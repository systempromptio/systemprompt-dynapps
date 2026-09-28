//! The pieces every admin table renders the same way.
//!
//! A sortable column header and a filter chip look identical on every listing
//! because they are the same control, so they are declared once here and the
//! pages fill them in. Seven pages had each written their own copy of the
//! sort header before this, which is how a column ends up sorting on one page
//! and not on another.
//!
//! The other two shared table pieces live in
//! [`crate::handlers::ssr::list_view`] rather than here, because they are
//! built by functions in that module: `Pagination`, which the
//! `components/pagination` partial reads, and `Chip`, the removable chip the
//! identity filter ribbon renders for a filter already in force.

use serde::Serialize;

/// One sortable column header, as `components/sort-header` reads it.
#[derive(Debug, Clone, Serialize)]
pub struct SortHeaderView {
    pub label: &'static str,
    pub class: &'static str,
    pub hint: &'static str,
    pub url: String,
    pub active: bool,
    pub aria_sort: &'static str,
    pub indicator: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct SortColumn {
    pub key: &'static str,
    pub label: &'static str,
    pub class: &'static str,
    pub hint: &'static str,
}

impl SortHeaderView {
    #[must_use]
    pub const fn next_dir(active: bool, descending: bool) -> &'static str {
        if active && descending { "asc" } else { "desc" }
    }

    #[must_use]
    pub const fn new(
        (label, class, hint): (&'static str, &'static str, &'static str),
        url: String,
        active: bool,
        descending: bool,
    ) -> Self {
        Self {
            label,
            class,
            hint,
            url,
            active,
            aria_sort: if !active {
                "none"
            } else if descending {
                "descending"
            } else {
                "ascending"
            },
            indicator: if !active {
                "\u{2195}"
            } else if descending {
                "\u{25bc}"
            } else {
                "\u{25b2}"
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FilterChipView {
    pub label: &'static str,
    pub href: String,
    pub is_active: bool,
    pub count: i64,
}
