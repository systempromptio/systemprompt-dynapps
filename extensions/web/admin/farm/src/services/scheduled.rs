//! A farm run the scheduler queues.
//!
//! The scheduler has no session, so it names the person the run launches as.
//! A farm in `run_as: {mode: user}` then spends that person's own connector
//! grants (dynapps-odoo, dynbrain) exactly as a console launch by them would;
//! the run records `triggered_via = scheduler`, which is how its report and
//! the farm page tell it from a manual run. An instance where the launcher
//! does not exist queues nothing: the services tree is shared by every
//! environment, and only production holds that person.
//!
//! With a `case`, the run is launched as that eval case (the public API's
//! case trigger), so the nightly run is scored like any evaluated run: the
//! case prompt and fixture apply, and the judge settles it when it ends.

use std::sync::Arc;

use crate::error::FarmResult;
use crate::repositories::farms;
use crate::repositories::rows::RunRow;
use crate::services::evals::{CaseLaunch, start_case_eval};
use crate::services::runs::{LaunchRequest, launch};
use crate::state::FarmState;
use crate::types::{FarmAgentId, FarmId, TriggeredVia};

#[derive(Debug, Clone)]
pub struct ScheduledLaunch {
    pub farm_id: FarmId,
    pub agent_id: FarmAgentId,
    pub launcher: String,
    pub case: Option<String>,
}

pub async fn launch_scheduled(
    state: &Arc<FarmState>,
    scheduled: ScheduledLaunch,
) -> FarmResult<Option<RunRow>> {
    let Some(user_id) =
        farms::find_launcher(state.pool.as_ref(), scheduled.launcher.trim()).await?
    else {
        return Ok(None);
    };
    let request = LaunchRequest {
        farm_id: scheduled.farm_id,
        agent_id: scheduled.agent_id,
        user_id,
        triggered_via: TriggeredVia::Scheduler,
        prompt: None,
        case_prompt: None,
        fixture: None,
        model: None,
        max_turns: None,
        max_budget_usd: None,
        timeout_seconds: None,
        eval_result_id: None,
        client_id: None,
    };
    match scheduled.case {
        Some(case_id) => {
            let judge = state.judge.judge_for(None, &request.user_id);
            let launched =
                start_case_eval(Arc::clone(state), judge, CaseLaunch { request, case_id }).await?;
            Ok(Some(launched.run))
        },
        None => launch(state, request).await.map(Some),
    }
}
