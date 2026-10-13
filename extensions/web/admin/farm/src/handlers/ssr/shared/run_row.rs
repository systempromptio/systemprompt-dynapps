//! One run as a table row, shared by the explorer, the overview board, the
//! farm and agent pages and the person drill-in. It reads the farm row only:
//! tool calls and events are the counters stamped when the run finished, so
//! a live row shows a dash until then.

use crate::types::FarmAgentId;
use chrono::Utc;
use serde::Serialize;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::ssr::format::{local_time, relative_time};
use systemprompt_web_shared::format::{format_cost, format_duration_ms, short_id};

use super::status::{BadgeView, is_live, status_badge, trigger_badge};
use crate::repositories::rows::RunRow;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RunRowView {
    pub run_id: String,
    pub run_id_short: String,
    pub detail_url: String,
    pub started_local: String,
    pub started_relative: String,
    pub farm_id: String,
    pub farm_url: String,
    pub agent_id: FarmAgentId,
    pub agent_url: String,
    pub user_id: UserId,
    pub user_url: String,
    pub trigger: BadgeView,
    pub status: BadgeView,
    pub is_live: bool,
    pub tool_calls: String,
    pub events_display: String,
    pub cost_display: String,
    pub duration_display: String,
    pub prompt_preview: String,
}

pub(crate) fn run_row_view(run: &RunRow) -> RunRowView {
    let started = run.started_at.unwrap_or(run.created_at);
    let live = is_live(&run.status);
    let duration_ms = match (run.started_at, run.finished_at) {
        (Some(s), Some(f)) => Some((f - s).num_milliseconds().max(0)),
        (Some(s), None) if live => Some((Utc::now() - s).num_milliseconds().max(0)),
        _ => None,
    };
    let counter = |n: i32| {
        if run.finished_at.is_some() {
            n.to_string()
        } else {
            "—".to_owned()
        }
    };
    RunRowView {
        run_id: run.id.as_str().to_owned(),
        run_id_short: short_id(run.id.as_str()),
        detail_url: format!("/admin/farm/runs/{}", run.id.as_str()),
        started_local: local_time(started),
        started_relative: relative_time(started),
        farm_id: run.farm_id.as_str().to_owned(),
        farm_url: format!("/admin/farm/{}", run.farm_id.as_str()),
        agent_id: run.agent_id.clone(),
        agent_url: format!(
            "/admin/farm/{}/agents/{}",
            run.farm_id.as_str(),
            run.agent_id.as_str()
        ),
        user_id: run.user_id.clone(),
        user_url: format!("/admin/users/{}", run.user_id.as_str()),
        trigger: trigger_badge(&run.triggered_via),
        status: status_badge(&run.status),
        is_live: live,
        tool_calls: counter(run.tool_call_count),
        events_display: counter(run.event_count),
        cost_display: if run.finished_at.is_some() || run.cost_microdollars > 0 {
            format_cost(run.cost_microdollars)
        } else {
            "—".to_owned()
        },
        duration_display: duration_ms.map_or_else(|| "—".to_owned(), format_duration_ms),
        prompt_preview: run.prompt.chars().take(120).collect(),
    }
}

pub(crate) fn run_row_views(runs: &[RunRow]) -> Vec<RunRowView> {
    runs.iter().map(run_row_view).collect()
}
