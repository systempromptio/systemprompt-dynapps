//! The links the conversations page draws from its query: window tabs,
//! breakdown tabs, pagination and sortable column headers.

use crate::handlers::ssr::list_view::{PageWindow, Pagination};
use crate::handlers::ssr::types::{SortHeaderView, TabLinkView};
use crate::repositories::analysis::conversations::{BreakdownBy, FactSort};

use super::query::ConversationAnalysisQuery;

impl ConversationAnalysisQuery {
    pub(crate) fn range_links(&self) -> Vec<TabLinkView> {
        let prefix = self.link_prefix(&["since", "preset", "page"]);
        let active = self.since_label();
        [
            ("24h", "24h"),
            ("7d", "7 days"),
            ("30d", "30 days"),
            ("90d", "90 days"),
            ("all", "All"),
        ]
        .iter()
        .map(|(slug, label)| TabLinkView {
            slug,
            label,
            href: format!("{prefix}since={slug}"),
            is_active: *slug == active,
            count: None,
        })
        .collect()
    }

    pub(crate) fn breakdown_tabs(&self) -> Vec<TabLinkView> {
        let prefix = self.link_prefix(&["by", "page"]);
        let active = self.breakdown();
        BreakdownBy::ALL
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

    pub(crate) fn pagination(&self, window: PageWindow) -> Pagination {
        let page = window.index;
        let prefix = self.link_prefix(&["page"]);
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
        col: FactSort,
        label: &'static str,
        class: &'static str,
        hint: &'static str,
    ) -> SortHeaderView {
        let prefix = self.link_prefix(&["sort", "dir", "page"]);
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
