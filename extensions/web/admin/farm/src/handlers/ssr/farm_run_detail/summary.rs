//! The KPI strip: what the run has cost and done so far. While the run is
//! live the finished-run columns are still empty, so turns, tokens and cost
//! fall back to the gateway requests it has already made.

use chrono::Utc;
use serde::Serialize;
use systemprompt_web_admin_core::ssr::format::{format_cost, format_duration_ms, short_num};

use crate::handlers::ssr::run_events_view::RunEventView;
use crate::handlers::ssr::shared::is_live;
use crate::repositories::rows::RunRow;
use crate::repositories::summaries::RunRequestRow;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RunSummaryView {
    pub is_live: bool,
    pub duration_ms: i64,
    pub duration_display: String,
    pub duration_note: &'static str,
    pub started_epoch_ms: Option<i64>,
    pub turns: String,
    pub tokens_in: String,
    pub tokens_out: String,
    pub cost_display: String,
    pub cost_note: String,
    pub event_count: usize,
    pub tool_calls: usize,
    pub errors: usize,
}

pub(crate) fn run_duration_ms(run: &RunRow) -> i64 {
    run.duration_ms
        .unwrap_or_else(|| match (run.started_at, run.finished_at) {
            (Some(s), Some(f)) => (f - s).num_milliseconds().max(0),
            (Some(s), None) if is_live(&run.status) => (Utc::now() - s).num_milliseconds().max(0),
            _ => 0,
        })
}

pub(crate) fn run_summary(
    run: &RunRow,
    events: &[RunEventView],
    requests: &[RunRequestRow],
) -> RunSummaryView {
    let live = is_live(&run.status);
    let duration_ms = run_duration_ms(run);
    let req_in: i64 = requests
        .iter()
        .filter_map(|r| r.input_tokens)
        .map(i64::from)
        .sum();
    let req_out: i64 = requests
        .iter()
        .filter_map(|r| r.output_tokens)
        .map(i64::from)
        .sum();
    let req_cost: i64 = requests.iter().map(|r| r.cost_microdollars).sum();
    let turns = run.num_turns.map_or_else(
        || i64::try_from(requests.len()).unwrap_or(i64::MAX),
        i64::from,
    );
    let cost = run.cost_microdollars.unwrap_or(req_cost);
    RunSummaryView {
        is_live: live,
        duration_ms,
        duration_display: if run.started_at.is_some() {
            format_duration_ms(duration_ms)
        } else {
            "—".to_owned()
        },
        duration_note: if live { "running" } else { "start to finish" },
        started_epoch_ms: run.started_at.map(|s| s.timestamp_millis()),
        turns: turns.to_string(),
        tokens_in: short_num(run.input_tokens.unwrap_or(req_in)),
        tokens_out: short_num(run.output_tokens.unwrap_or(req_out)),
        cost_display: format_cost(cost),
        cost_note: format!("{} gateway requests", requests.len()),
        event_count: events.len(),
        tool_calls: events.iter().filter(|e| e.is_tool_use).count(),
        errors: events.iter().filter(|e| e.is_error).count(),
    }
}
