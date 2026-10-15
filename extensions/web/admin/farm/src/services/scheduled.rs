//! A farm run the scheduler queues.
//!
//! The scheduler has no session, so it names the person the run launches as.
//! A farm in `run_as: {mode: user}` then spends that person's own connector
//! grants (dynapps-odoo, dynbrain) exactly as a console launch by them would;
//! the run records `triggered_via = scheduler`, which is how its report and
//! the farm page tell it from a manual run.

use std::sync::Arc;

use crate::error::{FarmError, FarmResult};
use crate::repositories::farms;
use crate::repositories::rows::RunRow;
use crate::services::runs::{LaunchRequest, launch};
use crate::state::FarmState;
use crate::types::{FarmAgentId, FarmId, TriggeredVia};

#[derive(Debug, Clone)]
pub struct ScheduledLaunch {
    pub farm_id: FarmId,
    pub agent_id: FarmAgentId,
    pub launcher: String,
}

pub async fn launch_scheduled(
    state: &Arc<FarmState>,
    scheduled: ScheduledLaunch,
) -> FarmResult<RunRow> {
    let user_id = farms::find_launcher(state.pool.as_ref(), scheduled.launcher.trim())
        .await?
        .ok_or_else(|| {
            FarmError::Config(format!(
                "scheduled launcher {:?} is not an active person",
                scheduled.launcher
            ))
        })?;
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
    launch(state, request).await
}
