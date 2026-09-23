//! The links the conversations page draws from its query: window tabs,
//! breakdown tabs, pagination and sortable column headers.

use crate::handlers::ssr::list_view::{PageWindow, Pagination};
use crate::handlers::ssr::types::{SortHeaderView, TabLinkView};
use crate::repositories::analysis::conversations::{BreakdownBy, FactSort};

use super::query::ConversationAnalysisQuery;
use crate::handlers::ssr::list_view::paginate_prefixed;

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
        paginate_prefixed(window, &self.link_prefix(&["page"]))
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
        let next_dir = SortHeaderView::next_dir(active, is_desc);
        SortHeaderView::new(
            (label, class, hint),
            format!("{prefix}sort={}&dir={next_dir}", col.as_str()),
            active,
            is_desc,
        )
    }
}
