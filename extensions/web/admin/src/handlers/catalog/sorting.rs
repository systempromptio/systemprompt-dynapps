//! One sortable-column builder for every platform list page.
//!
//! The catalog lists are small enough to sort in memory — they come from YAML
//! on disk, not from a query — so what varies between them is only the column
//! set and the comparator. This module owns the URL and ARIA half of that, so
//! adding a column to a page is naming it, not re-deriving the link, the
//! indicator glyph and the `aria-sort` value a fourth time.

pub(crate) use crate::handlers::ssr::types::SortHeaderView;
pub use crate::handlers::ssr::types::table::SortColumn;

pub fn sort_headers(
    base_url: &str,
    columns: &[SortColumn],
    active_key: &str,
    active_dir: &str,
    preserved: &str,
) -> Vec<SortHeaderView> {
    let prefix = if preserved.is_empty() {
        format!("{base_url}?")
    } else {
        format!("{base_url}?{preserved}&")
    };
    columns
        .iter()
        .map(|col| {
            let active = col.key == active_key;
            let descending = active_dir != "asc";
            let next_dir = SortHeaderView::next_dir(active, descending);
            SortHeaderView::new(
                (col.label, col.class, col.hint),
                format!("{prefix}sort={}&dir={next_dir}", col.key),
                active,
                descending,
            )
        })
        .collect()
}

pub fn direction(raw: Option<&str>) -> &'static str {
    if raw == Some("asc") { "asc" } else { "desc" }
}

pub fn preserved_search(q: &str) -> String {
    if q.is_empty() {
        return String::new();
    }
    format!("q={}", urlencoding::encode(q))
}

pub fn matches(haystack: &[&str], needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    let needle = needle.to_lowercase();
    haystack
        .iter()
        .any(|field| field.to_lowercase().contains(&needle))
}

pub(crate) fn apply_direction<T>(
    rows: &mut [T],
    dir: &str,
    cmp: impl Fn(&T, &T) -> std::cmp::Ordering,
) {
    rows.sort_by(|a, b| {
        let ord = cmp(a, b);
        if dir == "asc" { ord } else { ord.reverse() }
    });
}
