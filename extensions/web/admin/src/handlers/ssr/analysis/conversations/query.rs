//! The query string `/admin/analysis/conversations` binds, and the links that
//! carry it: every filter, the breakdown dimension, sort and page.

use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use systemprompt::identifiers::UserId;
use urlencoding::encode as urlencode;

use crate::handlers::ssr::list_view::{PageWindow, Pagination};
use crate::handlers::ssr::types::{SortHeaderView, TabLinkView};
use crate::repositories::analysis::conversations::{BreakdownBy, ScoredFilter};
use crate::repositories::analytics::conversation_rows::ConversationSort;

pub(crate) const BASE_URL: &str = "/admin/analysis/conversations";
pub(crate) const PAGE_SIZE: i64 = 50;

#[derive(Debug, Deserialize, Default)]
pub(crate) struct ConversationAnalysisQuery {
    pub group: Option<String>,
    pub project: Option<String>,
    pub user_id: Option<UserId>,
    pub category: Option<String>,
    pub outcome: Option<String>,
    pub skill: Option<String>,
    pub scored: Option<String>,
    pub q: Option<String>,
    pub since: Option<String>,
    // Why: the shell's scope bar emits `?preset=`; read as a fallback so it
    // is not silently ignored.
    pub preset: Option<String>,
    pub by: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub page: Option<i64>,
}

impl ConversationAnalysisQuery {
    fn trimmed(value: Option<&str>) -> Option<&str> {
        value.map(str::trim).filter(|v| !v.is_empty())
    }

    pub(super) fn since_label(&self) -> &'static str {
        let raw =
            Self::trimmed(self.since.as_deref()).or_else(|| Self::trimmed(self.preset.as_deref()));
        match raw {
            Some("1d" | "24h") => "24h",
            Some("7d") => "7d",
            Some("90d") => "90d",
            Some("all") => "all",
            _ => "30d",
        }
    }

    pub(super) fn since_datetime(&self) -> Option<DateTime<Utc>> {
        let dur = match self.since_label() {
            "24h" => Duration::hours(24),
            "7d" => Duration::days(7),
            "30d" => Duration::days(30),
            "90d" => Duration::days(90),
            _ => return None,
        };
        Some(Utc::now() - dur)
    }

    pub(super) fn user_id(&self) -> Option<UserId> {
        self.user_id
            .clone()
            .filter(|u| !u.as_str().trim().is_empty())
    }

    pub(super) fn category(&self) -> Option<String> {
        Self::trimmed(self.category.as_deref()).map(str::to_owned)
    }

    pub(super) fn outcome(&self) -> Option<String> {
        Self::trimmed(self.outcome.as_deref()).map(str::to_owned)
    }

    pub(super) fn skill(&self) -> Option<String> {
        Self::trimmed(self.skill.as_deref()).map(str::to_owned)
    }

    pub(super) fn scored(&self) -> Option<ScoredFilter> {
        ScoredFilter::parse(Self::trimmed(self.scored.as_deref()))
    }

    pub(super) fn free_text(&self) -> Option<String> {
        Self::trimmed(self.q.as_deref()).map(str::to_owned)
    }

    pub(super) fn breakdown(&self) -> BreakdownBy {
        BreakdownBy::from_breakdown_param(self.by.as_deref())
    }

    pub(super) fn sort(&self) -> ConversationSort {
        ConversationSort::parse_conversation_sort(self.sort.as_deref())
    }

    pub(super) fn descending(&self) -> bool {
        self.dir.as_deref() != Some("asc")
    }

    pub(super) fn page(&self) -> i64 {
        self.page.unwrap_or(0).max(0)
    }

    fn pairs(&self) -> [(&'static str, Option<&str>); 13] {
        [
            ("group", self.group.as_deref()),
            ("project", self.project.as_deref()),
            ("user_id", self.user_id.as_ref().map(UserId::as_str)),
            ("category", self.category.as_deref()),
            ("outcome", self.outcome.as_deref()),
            ("skill", self.skill.as_deref()),
            ("scored", self.scored.as_deref()),
            ("q", self.q.as_deref()),
            ("since", self.since.as_deref()),
            ("preset", self.preset.as_deref()),
            ("by", self.by.as_deref()),
            ("sort", self.sort.as_deref()),
            ("dir", self.dir.as_deref()),
        ]
    }

    pub(super) fn preserved(&self, drop: &[&str]) -> Vec<(String, String)> {
        self.pairs()
            .iter()
            .filter(|(name, _)| !drop.contains(name))
            .filter_map(|(name, val)| {
                val.map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(|v| ((*name).to_owned(), v.to_owned()))
            })
            .collect()
    }

    pub(super) fn link_prefix(&self, drop: &[&str]) -> String {
        let qs = self
            .preserved(drop)
            .iter()
            .map(|(name, value)| format!("{name}={}", urlencode(value)))
            .collect::<Vec<_>>()
            .join("&");
        if qs.is_empty() {
            format!("{BASE_URL}?")
        } else {
            format!("{BASE_URL}?{qs}&")
        }
    }

    // Why: the link a breakdown row offers — the current filters narrowed by
    // one more `name=value`, back on page one.
    pub(super) fn narrowed(&self, name: &str, value: &str) -> String {
        format!(
            "{}{name}={}",
            self.link_prefix(&[name, "page"]),
            urlencode(value)
        )
    }

    pub(super) fn breakdown_tabs(&self) -> Vec<TabLinkView> {
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

    pub(super) fn pagination(&self, window: PageWindow) -> Pagination {
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

    pub(super) fn sort_header(
        &self,
        col: ConversationSort,
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
