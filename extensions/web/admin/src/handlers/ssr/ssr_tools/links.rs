//! The navigation the Tools and Artifacts pages hang off their query: the
//! window tabs, the breakdown tabs, the pager and the sortable headers.

use super::query::{Lens, ToolsQuery};
use crate::handlers::ssr::list_view::{PageWindow, Pagination};
use crate::handlers::ssr::types::{SortHeaderView, TabLinkView};
use crate::repositories::analysis::tools::{ToolBreakdownBy, ToolSort};
use crate::util::time_range::TimeRange;

impl ToolsQuery {
    pub(crate) fn range_links(&self, lens: Lens, range: TimeRange) -> Vec<TabLinkView> {
        let prefix = self.link_prefix(lens, &["preset", "from", "to", "page"]);
        let active = self.preset_label(range);
        [
            ("1h", "1h"),
            ("24h", "24h"),
            ("7d", "7 days"),
            ("30d", "30 days"),
        ]
        .iter()
        .map(|(slug, label)| TabLinkView {
            slug,
            label,
            href: format!("{prefix}preset={slug}"),
            is_active: *slug == active,
            count: None,
        })
        .collect()
    }

    pub(crate) fn breakdown_tabs(&self, lens: Lens) -> Vec<TabLinkView> {
        let prefix = self.link_prefix(lens, &["by", "page"]);
        let active = self.breakdown(lens);
        ToolBreakdownBy::ALL
            .iter()
            .map(|by| TabLinkView {
                slug: by.as_str(),
                label: by.label(),
                href: format!("{prefix}by={}", by.as_str()),
                is_active: *by == active,
                count: None,
            })
            .collect()
    }

    pub(crate) fn pagination(&self, lens: Lens, window: PageWindow) -> Pagination {
        let page = window.index;
        let prefix = self.link_prefix(lens, &["page"]);
        let prev_url = (page > 0).then(|| format!("{prefix}page={}", page.saturating_sub(1)));
        let next_url =
            (page + 1 < window.total_pages).then(|| format!("{prefix}page={}", page + 1));
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

    pub(crate) fn sort_header(
        &self,
        lens: Lens,
        col: ToolSort,
        label: &'static str,
        class: &'static str,
        hint: &'static str,
    ) -> SortHeaderView {
        let prefix = self.link_prefix(lens, &["sort", "dir", "page"]);
        let active = col == self.sort();
        let is_desc = self.descending();
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
    }
}
