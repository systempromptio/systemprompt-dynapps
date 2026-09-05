//! Typed template context for the sessions list (`sessions.hbs`).

use serde::Serialize;
use systemprompt::identifiers::{SessionId, UserId};

use crate::handlers::ssr::list_view::{
use crate::handlers::ssr::types::SortHeaderView;
    AnnotatedOption, Chip, Pagination, Preserved, ScopeFilterView, TimeRangeContext,
};

#[derive(Debug, Serialize)]
pub(super) struct SessionsListPageContext {
    pub(super) page: &'static str,
    pub(super) title: &'static str,
    pub(super) current: CurrentSessionView,
    pub(super) time_range: TimeRangeContext,
    pub(super) filter_ribbon: FilterRibbon,
    pub(super) scope_filter: ScopeFilterView,
    pub(super) stats: StatsView,
    pub(super) sessions: Vec<SessionRowView>,
    pub(super) has_sessions: bool,
    pub(super) total_count: i64,
    pub(super) count_label: String,
    pub(super) pagination: Pagination,
    pub(super) sort_headers: SessionsSortHeaders,
    pub(super) error_only: bool,
    // Why: Link that toggles the errors-only filter on or off.
    pub(super) error_toggle_url: String,
}

// Why: The "you are here" strip: who this browser is signed in as, and a way
// into that session's own detail page.
#[derive(Debug, Serialize)]
pub(super) struct CurrentSessionView {
    pub(super) username: String,
    pub(super) session_id: Option<SessionId>,
    pub(super) session_id_short: Option<String>,
    pub(super) session_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct FilterRibbon {
    pub(super) base_url: &'static str,
    pub(super) preserved: Vec<Preserved>,
    pub(super) options: SessionFilterOptionsView,
    pub(super) chips: Vec<Chip>,
}

// Why: Only the user facet is offered — agent, policy and decision are
// properties of a trace, not of the session that contains it.
#[derive(Debug, Default, Serialize)]
pub(super) struct SessionFilterOptionsView {
    pub(super) users: Vec<AnnotatedOption>,
}

#[derive(Debug, Serialize)]
pub(super) struct StatsView {
    pub(super) total_sessions: i64,
    pub(super) error_sessions: i64,
    pub(super) total_requests: i64,
    pub(super) total_tool_uses: i64,
    pub(super) tokens_display: String,
    pub(super) cost_display: String,
    pub(super) sessions_delta: Option<String>,
    pub(super) sessions_delta_dir: Option<&'static str>,
    pub(super) requests_delta: Option<String>,
    pub(super) requests_delta_dir: Option<&'static str>,
    pub(super) tokens_delta: Option<String>,
    pub(super) tokens_delta_dir: Option<&'static str>,
    pub(super) cost_delta: Option<String>,
    pub(super) cost_delta_dir: Option<&'static str>,
}

#[derive(Debug, Serialize)]
pub(super) struct SessionsSortHeaders {
    pub(super) started: SortHeaderView,
    pub(super) duration: SortHeaderView,
    pub(super) requests: SortHeaderView,
    pub(super) tokens: SortHeaderView,
    pub(super) cost: SortHeaderView,
}

#[derive(Debug, Serialize)]
pub(super) struct SessionRowView {
    pub(super) session_id: SessionId,
    pub(super) session_id_short: String,
    pub(super) detail_url: String,
    pub(super) ai_title: Option<String>,
    pub(super) user_id: Option<UserId>,
    pub(super) user_label: String,
    pub(super) user_url: Option<String>,
    // Why: the primary group and project exclusive attribution files this
    // person's usage under — "unattributed" when no default row covers them,
    // which is a bucket the page shows rather than a blank it hides.
    pub(super) group_label: String,
    pub(super) project_label: String,
    pub(super) source_label: &'static str,
    pub(super) source_variant: &'static str,
    pub(super) model: Option<String>,
    pub(super) client_source: Option<String>,
    pub(super) request_count: i64,
    pub(super) context_count: i64,
    pub(super) trace_count: i64,
    pub(super) tool_uses: i64,
    pub(super) tokens_display: String,
    pub(super) cost_display: String,
    pub(super) duration_display: String,
    pub(super) started_at: Option<String>,
    pub(super) started_at_local: Option<String>,
    pub(super) error_count: i64,
    pub(super) has_error: bool,
    pub(super) status_label: String,
}
