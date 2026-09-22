//! The date window the history listings read: a preset number of days, a
//! custom `[start, end)` range, or all time.
//!
//! A conversation falls in the window when its last activity does, so a long
//! conversation still running today is listed under "7 days".

use chrono::{DateTime, Duration, Utc};

use crate::export::window::next_midnight;
use crate::handlers::ssr::analysis::time;
use crate::handlers::ssr::types::TabLinkView;

use super::HistoryQuery;

const PRESETS: [(u32, &str, &str); 4] = [
    (7, "7", "7 days"),
    (30, "30", "30 days"),
    (90, "90", "90 days"),
    (365, "365", "1 year"),
];

#[derive(Debug, Clone, Copy)]
pub(super) enum HistoryWindow {
    All,
    Days(u32),
    Custom {
        since: DateTime<Utc>,
        until: DateTime<Utc>,
    },
}

impl HistoryWindow {
    // Why: the page is a browse surface, so a half-given or unparseable range
    // falls back to the preset (or all time) rather than failing the page;
    // the export resolves its own window strictly.
    pub(super) fn of(query: &HistoryQuery) -> Self {
        let parsed = |v: Option<&str>| {
            v.map(str::trim)
                .filter(|v| !v.is_empty())
                .and_then(time::parse)
        };
        match (parsed(query.start.as_deref()), parsed(query.end.as_deref())) {
            (Some(since), Some(until)) if since < until => Self::Custom { since, until },
            _ => match query.days {
                Some(days) if PRESETS.iter().any(|(d, ..)| *d == days) => Self::Days(days),
                _ => Self::All,
            },
        }
    }

    pub(super) fn bounds(self) -> (Option<DateTime<Utc>>, Option<DateTime<Utc>>) {
        match self {
            Self::All => (None, None),
            Self::Days(days) => {
                let until = next_midnight();
                (Some(until - Duration::days(i64::from(days))), Some(until))
            },
            Self::Custom { since, until } => (Some(since), Some(until)),
        }
    }

    pub(super) fn label(self) -> String {
        match self {
            Self::All => "all time".to_owned(),
            Self::Days(365) => "last year".to_owned(),
            Self::Days(days) => format!("last {days} days"),
            Self::Custom { since, until } => {
                format!("{} to {}", time::render(since), time::render(until))
            },
        }
    }

    /// The query parameters that carry this window; all time carries none.
    pub(super) fn pairs(self) -> Vec<(&'static str, String)> {
        match self {
            Self::All => Vec::new(),
            Self::Days(days) => vec![("days", days.to_string())],
            Self::Custom { since, until } => {
                vec![("start", time::render(since)), ("end", time::render(until))]
            },
        }
    }

    /// The `key=value` pairs that carry this window on a link.
    pub(super) fn query_parts(self) -> Vec<String> {
        self.pairs()
            .into_iter()
            .map(|(key, value)| format!("{key}={}", urlencoding::encode(&value)))
            .collect()
    }

    // Why: the export dialog has no "all time" — its retained contract tops
    // out at a year — so an unbounded page opens the dialog on its widest.
    pub(super) fn export_pairs(self) -> Vec<(&'static str, String)> {
        match self {
            Self::All => vec![("days", "365".to_owned())],
            Self::Days(_) | Self::Custom { .. } => self.pairs(),
        }
    }

    /// The window tabs; `prefix` already carries every other filter and
    /// ends in `?` or `&`.
    pub(super) fn tabs(self, prefix: &str) -> Vec<TabLinkView> {
        let active = match self {
            Self::Days(days) => Some(days),
            Self::All | Self::Custom { .. } => None,
        };
        let all_href = prefix.trim_end_matches(['?', '&']).to_owned();
        PRESETS
            .iter()
            .map(|&(days, slug, label)| TabLinkView {
                slug,
                label,
                href: format!("{prefix}days={days}"),
                is_active: active == Some(days),
                count: None,
            })
            .chain(std::iter::once(TabLinkView {
                slug: "all",
                label: "All",
                href: all_href,
                is_active: matches!(self, Self::All),
                count: None,
            }))
            .collect()
    }
}
