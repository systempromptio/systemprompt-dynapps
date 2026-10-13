//! The KPI strip: what the run has cost and done so far. The executor's
//! figures (duration, turns, tokens) live on the `agent_runs` row and are
//! empty until the run finishes, so while it is live they fall back to the
//! gateway requests it has already made.

use chrono::Utc;
use serde::Serialize;
use systemprompt_web_admin_core::ssr::format::{format_cost, format_duration_ms, short_num};

use super::RunData;
use crate::handlers::ssr::shared::is_live;

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
    pub event_count: i64,
    pub tool_calls: i64,
    pub errors: usize,
}

pub(crate) fn run_duration_ms(data: &RunData) -> i64 {
    let started = data.agent_run.started_at.or(data.run.started_at);
    let finished = data.agent_run.finished_at.or(data.run.finished_at);
    data.agent_run
        .duration_ms
        .unwrap_or_else(|| match (started, finished) {
            (Some(s), Some(f)) => (f - s).num_milliseconds().max(0),
            (Some(s), None) if is_live(&data.run.status) => {
                (Utc::now() - s).num_milliseconds().max(0)
            },
            _ => 0,
        })
}

pub(crate) fn run_summary(data: &RunData) -> RunSummaryView {
    let run = &data.run;
    let agent_run = &data.agent_run;
    let requests = &data.requests;
    let live = is_live(&run.status);
    let duration_ms = run_duration_ms(data);
    let started = agent_run.started_at.or(run.started_at);
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
    let turns = agent_run.turns.map_or_else(
        || i64::try_from(requests.len()).unwrap_or(i64::MAX),
        i64::from,
    );
    let cost = agent_run
        .cost_microdollars
        .filter(|c| *c > 0)
        .or_else(|| (run.cost_microdollars > 0).then_some(run.cost_microdollars))
        .unwrap_or(req_cost);
    let request_count = if live {
        i64::try_from(requests.len()).unwrap_or(i64::MAX)
    } else {
        i64::from(run.gateway_request_count)
    };
    RunSummaryView {
        is_live: live,
        duration_ms,
        duration_display: if started.is_some() {
            format_duration_ms(duration_ms)
        } else {
            "—".to_owned()
        },
        duration_note: if live { "running" } else { "start to finish" },
        started_epoch_ms: started.map(|s| s.timestamp_millis()),
        turns: turns.to_string(),
        tokens_in: short_num(agent_run.input_tokens.unwrap_or(req_in)),
        tokens_out: short_num(agent_run.output_tokens.unwrap_or(req_out)),
        cost_display: format_cost(cost),
        cost_note: format!("{request_count} gateway requests"),
        event_count: data.event_count,
        tool_calls: data.tool_call_count,
        errors: data.events.iter().filter(|e| e.is_error).count(),
    }
}
