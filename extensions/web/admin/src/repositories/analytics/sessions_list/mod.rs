//! Sessions-list repository — drives `/admin/sessions`.
//!
//! A session id is written by two producers that never meet: the gateway
//! stamps it on every `ai_requests` row, and the hook pipeline rolls its
//! events up into `plugin_session_summaries`. Neither table is a superset of
//! the other, so both are `FULL OUTER JOIN`ed on `session_id` — the same shape
//! `session_detail::find_session_header` uses, which is what makes a row here
//! resolve to a detail page rather than a 404.
//!
//! Every read is narrowed by the caller's resolved
//! [`SubjectScope`](crate::repositories::scope::SubjectScope), so a group or
//! project selection reaches the rows and the KPI strip above them alike.

use chrono::{DateTime, Utc};
use systemprompt::identifiers::{PluginId, SessionId, UserId};

mod kpis;
mod list;

pub use kpis::{SessionListKpis, get_session_list_kpis};
pub use list::list_sessions_paged;

/// Narrowing applied to both the list and the KPI strip, so the numbers above
/// the table always describe the rows inside it.
#[derive(Debug, Clone, Default)]
pub struct SessionListFilter {
    pub user_id: Option<UserId>,
    /// The caller's resolved `SubjectScope::as_sql()`; `None` = every user.
    pub subject_ids: Option<Vec<String>>,
    /// Only sessions that recorded a failed request or a hook error.
    pub error_only: bool,
}

/// The five columns the list can be ordered by, each in both directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionSortColumn {
    StartedAt,
    Duration,
    Requests,
    Tokens,
    Cost,
}

impl SessionSortColumn {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StartedAt => "started_at",
            Self::Duration => "duration",
            Self::Requests => "requests",
            Self::Tokens => "tokens",
            Self::Cost => "cost",
        }
    }

    #[must_use]
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            Some("duration") => Self::Duration,
            Some("requests") => Self::Requests,
            Some("tokens") => Self::Tokens,
            Some("cost") => Self::Cost,
            _ => Self::StartedAt,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionSortDir {
    Asc,
    Desc,
}

impl SessionSortDir {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }

    #[must_use]
    pub fn parse(value: Option<&str>) -> Self {
        if value == Some("asc") {
            Self::Asc
        } else {
            Self::Desc
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SessionSort {
    pub column: SessionSortColumn,
    pub dir: SessionSortDir,
}

impl Default for SessionSort {
    fn default() -> Self {
        Self {
            column: SessionSortColumn::StartedAt,
            dir: SessionSortDir::Desc,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SessionPage {
    pub sort: SessionSort,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone)]
pub struct SessionListItem {
    pub session_id: SessionId,
    pub user_id: Option<UserId>,
    pub display_name: Option<String>,
    /// Primary group the person is attributed to, for the Group column.
    pub group_name: Option<String>,
    /// Primary project the person is attributed to, for the Project column.
    pub project_name: Option<String>,
    pub model: Option<String>,
    /// Hook-generated summary title, when the session was summarised.
    pub ai_title: Option<String>,
    pub plugin_id: Option<PluginId>,
    pub client_source: Option<String>,
    pub status: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub request_count: i64,
    pub context_count: i64,
    pub trace_count: i64,
    pub tool_uses: i64,
    /// Failed gateway requests plus hook-reported errors.
    pub error_count: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost_microdollars: i64,
    /// The session produced gateway traffic (`ai_requests`).
    pub has_gateway: bool,
    /// The session produced hook events (`plugin_session_summaries`).
    pub has_hooks: bool,
}
