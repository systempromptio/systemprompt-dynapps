//! The runs explorer's URL contract: the query both the page and its live
//! fragment read, the filter and sort it maps to, and the query strings the
//! chips, sort headers and pagination rebuild from it.

use chrono::{DateTime, Utc};
use serde::Deserialize;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::ssr::list_view::query_string_dropping;

use crate::repositories::filters::{RunFilter, RunSort};
use crate::types::{FarmAgentId, FarmId, RunStatus, TriggeredVia};

pub(crate) const BASE_URL: &str = "/admin/farm/runs";
pub(crate) const FRAGMENT_URL: &str = "/admin/api/farm/runs";

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RunsQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
    pub farm_id: Option<String>,
    #[serde(rename = "agent_id")]
    pub agent_param: Option<String>,
    #[serde(rename = "user_id")]
    pub user_param: Option<String>,
    pub status: Option<String>,
    pub trigger: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub page: Option<i64>,
}

fn present(value: Option<&String>) -> Option<&str> {
    value.map(String::as_str).filter(|v| !v.is_empty())
}

impl RunsQuery {
    #[must_use]
    pub fn farm(&self) -> Option<&str> {
        present(self.farm_id.as_ref())
    }

    #[must_use]
    pub fn agent(&self) -> Option<&str> {
        present(self.agent_param.as_ref())
    }

    #[must_use]
    pub fn user(&self) -> Option<&str> {
        present(self.user_param.as_ref())
    }

    #[must_use]
    pub fn status_value(&self) -> Option<RunStatus> {
        present(self.status.as_ref()).and_then(RunStatus::parse)
    }

    #[must_use]
    pub fn trigger_value(&self) -> Option<TriggeredVia> {
        present(self.trigger.as_ref()).and_then(TriggeredVia::parse)
    }

    #[must_use]
    pub fn page_index(&self) -> i64 {
        self.page.unwrap_or(0).max(0)
    }

    fn pairs(&self) -> [(&'static str, Option<&str>); 10] {
        [
            ("preset", present(self.preset.as_ref())),
            ("from", present(self.from.as_ref())),
            ("to", present(self.to.as_ref())),
            ("farm_id", self.farm()),
            ("agent_id", self.agent()),
            ("user_id", self.user()),
            ("status", self.status_value().map(RunStatus::as_str)),
            ("trigger", self.trigger_value().map(TriggeredVia::as_str)),
            ("sort", present(self.sort.as_ref())),
            ("dir", present(self.dir.as_ref())),
        ]
    }

    #[must_use]
    pub fn query_string(&self, drop: &[&str]) -> String {
        query_string_dropping(&self.pairs(), &[], drop)
    }

    #[must_use]
    pub fn url_prefix(&self, drop: &[&str]) -> String {
        let qs = self.query_string(drop);
        if qs.is_empty() {
            format!("{BASE_URL}?")
        } else {
            format!("{BASE_URL}?{qs}&")
        }
    }

    #[must_use]
    pub fn url_without(&self, drop: &[&str]) -> String {
        let qs = self.query_string(drop);
        if qs.is_empty() {
            BASE_URL.to_owned()
        } else {
            format!("{BASE_URL}?{qs}")
        }
    }
}

#[must_use]
pub fn run_filter(query: &RunsQuery, since: DateTime<Utc>, until: DateTime<Utc>) -> RunFilter {
    RunFilter {
        farm_id: query.farm().map(FarmId::new),
        agent_id: query.agent().map(FarmAgentId::new),
        user_id: query.user().map(UserId::new),
        status: query.status_value(),
        triggered_via: query.trigger_value(),
        since: Some(since),
        until: Some(until),
    }
}

/// The sortable columns and the order each one reads in. Only the start time
/// sorts both ways; duration, tool calls and cost read largest first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunSortColumn {
    Started,
    Duration,
    ToolCalls,
    Cost,
}

impl RunSortColumn {
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::Duration => "duration",
            Self::ToolCalls => "tool_calls",
            Self::Cost => "cost",
        }
    }

    #[must_use]
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            Some("duration") => Self::Duration,
            Some("tool_calls" | "turns") => Self::ToolCalls,
            Some("cost") => Self::Cost,
            _ => Self::Started,
        }
    }
}

#[must_use]
pub fn run_sort(query: &RunsQuery) -> (RunSortColumn, RunSort) {
    let column = RunSortColumn::parse(query.sort.as_deref());
    let sort = match column {
        RunSortColumn::Started if query.dir.as_deref() == Some("asc") => RunSort::CreatedAsc,
        RunSortColumn::Started => RunSort::CreatedDesc,
        RunSortColumn::Duration => RunSort::DurationDesc,
        RunSortColumn::ToolCalls => RunSort::ToolCallsDesc,
        RunSortColumn::Cost => RunSort::CostDesc,
    };
    (column, sort)
}
