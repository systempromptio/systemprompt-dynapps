//! Stamps a finished `agent_runs` row's facts onto its `agent_farm_runs` row.
//!
//! [`observe_finished`] runs after the handler finishes a run, and
//! [`reconcile`] sweeps the farm's active rows for runs that turned terminal
//! without a handler: a queued run cancelled before it was claimed, or a run
//! whose lease expired on its final attempt (core marks those failed during
//! stale reclaim). Both are idempotent, since a farm row that is already
//! terminal is left alone.
//!
//! `finish_farm_run` stamps the same facts from a `RunOutcome` in hand, over
//! a pool, with no executor. `observe_finished` returns `false` when the agent
//! run is not terminal yet or the farm row was already stamped.
//! `unexecuted_outcome` is the outcome of a run the executor never ran:
//! cancelled while queued, or failed before the host started.
//!
//! Evals need no trigger: an eval case waits on the farm row's status, which
//! this stamp is what moves.

use std::str::FromStr;
use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::agent_runs::{AgentRunRepository, AgentRunRow, RunOutcome};
use systemprompt::identifiers::{AgentRunId, HostSessionId};

use crate::error::FarmResult;
use crate::repositories::filters::RunFinish;
use crate::repositories::rows::RunRow;
use crate::repositories::runs;
use crate::state::FarmState;
use crate::types::{AgentRunStatus, ErrorClass, FarmRunId, RunEventKind, RunStatus};

const EVENT_PAGE: i64 = 1_000;

fn agent_runs(pool: &Arc<PgPool>) -> AgentRunRepository {
    AgentRunRepository::from_pool(Arc::clone(pool))
}

fn count_i32(count: i64) -> i32 {
    i32::try_from(count).unwrap_or(i32::MAX)
}

#[must_use]
pub fn unexecuted_outcome(status: AgentRunStatus, class: ErrorClass, error: &str) -> RunOutcome {
    RunOutcome {
        status,
        error_class: class,
        session_id: None,
        cost_microdollars: 0,
        input_tokens: 0,
        output_tokens: 0,
        turns: 0,
        duration_ms: 0,
        structured_output: None,
        exit_code: None,
        error: Some(error.to_owned()),
        final_text: None,
    }
}

pub async fn observe_finished(state: &FarmState, agent_run_id: &AgentRunId) -> FarmResult<bool> {
    let Some(agent_run) = agent_runs(&state.pool).find_run(agent_run_id).await? else {
        return Ok(false);
    };
    let status = AgentRunStatus::from_str(&agent_run.status)?;
    if !status.is_terminal() {
        return Ok(false);
    }
    let pool = state.pool.as_ref();
    let Some(farm_run) = runs::find_run_by_agent_run(pool, agent_run_id).await? else {
        return Ok(false);
    };
    let facts = Facts {
        status,
        error_class: agent_run.error_class.clone(),
        cost_microdollars: agent_run.cost_microdollars.unwrap_or(0),
        host_session_id: agent_run.host_session_id.as_ref(),
    };
    stamp(&state.pool, &farm_run, &agent_run, facts).await
}

pub async fn finish_farm_run(
    pool: &Arc<PgPool>,
    run_id: &FarmRunId,
    outcome: &RunOutcome,
) -> FarmResult<bool> {
    let Some((farm_run, agent_run)) = runs::find_run_with_agent_run(pool, run_id).await? else {
        return Ok(false);
    };
    let facts = Facts {
        status: outcome.status,
        error_class: Some(outcome.error_class.as_str().to_owned()),
        cost_microdollars: outcome.cost_microdollars,
        host_session_id: outcome
            .session_id
            .as_ref()
            .or(agent_run.host_session_id.as_ref()),
    };
    stamp(pool, &farm_run, &agent_run, facts).await
}

struct Facts<'a> {
    status: AgentRunStatus,
    error_class: Option<String>,
    cost_microdollars: i64,
    host_session_id: Option<&'a HostSessionId>,
}

async fn stamp(
    pool: &Arc<PgPool>,
    farm_run: &RunRow,
    agent_run: &AgentRunRow,
    facts: Facts<'_>,
) -> FarmResult<bool> {
    if RunStatus::parse(&farm_run.status).is_some_and(RunStatus::is_terminal) {
        return Ok(false);
    }
    let repo = agent_runs(pool);
    let gateway_request_count = match agent_run.api_key_id.as_ref() {
        Some(key) => count_i32(runs::count_run_requests(pool, key).await?),
        None => 0,
    };
    let context_id = match facts.host_session_id {
        Some(session) => runs::find_run_context_id(pool, &agent_run.user_id, session).await?,
        None => None,
    };
    let finish = RunFinish {
        status: RunStatus::from_agent_run(facts.status),
        attempt: agent_run.attempt,
        error_class: facts.error_class,
        cost_microdollars: facts.cost_microdollars,
        event_count: count_i32(repo.count_events(&agent_run.id).await?),
        tool_call_count: count_tool_calls(&repo, &agent_run.id).await?,
        gateway_request_count,
        context_id,
    };
    Ok(runs::update_run_finished(pool, &farm_run.id, &finish).await?)
}

async fn count_tool_calls(repo: &AgentRunRepository, id: &AgentRunId) -> FarmResult<i32> {
    let mut after = -1;
    let mut count = 0i64;
    loop {
        let page = repo.list_events_after(id, after, EVENT_PAGE).await?;
        let Some(last) = page.last() else {
            return Ok(count_i32(count));
        };
        after = last.seq;
        let tool_uses = page
            .iter()
            .filter(|event| event.kind == RunEventKind::ToolUse)
            .count();
        count += i64::try_from(tool_uses).unwrap_or(i64::MAX);
    }
}

pub async fn reconcile(state: &FarmState) -> FarmResult<usize> {
    let mut stamped = 0;
    for run in runs::list_active_runs(state.pool.as_ref()).await? {
        if observe_finished(state, &run.agent_run_id).await? {
            stamped += 1;
        }
    }
    Ok(stamped)
}
