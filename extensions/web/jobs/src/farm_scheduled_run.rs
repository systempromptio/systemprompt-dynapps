//! `farm_scheduled_run` job: queues one agent-farm run on core's
//! `agent_runs` on the scheduler's behalf.
//!
//! The scheduler entry names the farm, the agent and the person the run
//! launches as (`farm`, `agent`, `launcher` parameters). The run is recorded
//! with `triggered_via = scheduler`; the farm's runner service claims it like
//! any other queued run. The job only queues: it never waits for the run.

use std::sync::Arc;

use systemprompt::config::AppPaths;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};
use systemprompt_web_admin_farm::services::scheduled::{ScheduledLaunch, launch_scheduled};
use systemprompt_web_admin_farm::state::build_farm_state_from_app;
use systemprompt_web_admin_farm::types::{FarmAgentId, FarmId};

use crate::error::JobError;

#[derive(Debug, Clone, Copy, Default)]
pub struct FarmScheduledRunJob;

fn required(ctx: &JobContext, key: &str) -> Result<String, JobError> {
    ctx.get_parameter_parsed::<String>(key)?
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| JobError::config(format!("farm_scheduled_run needs the `{key}` parameter")))
}

impl FarmScheduledRunJob {
    async fn queue(ctx: &JobContext) -> Result<JobResult, JobError> {
        let scheduled = ScheduledLaunch {
            farm_id: FarmId::new(required(ctx, "farm")?),
            agent_id: FarmAgentId::new(required(ctx, "agent")?),
            launcher: required(ctx, "launcher")?,
        };
        let db = ctx.get::<DbPool>()?;
        let paths = ctx.get::<Arc<AppPaths>>().ok().map(Arc::clone);
        let state = build_farm_state_from_app(db.write_pool(), paths.as_deref());
        let run = launch_scheduled(&state, scheduled)
            .await
            .map_err(|error| JobError::other(error.to_string()))?;
        tracing::info!(actor = %ctx.actor().user_id, run = run.id.as_str(),
            farm = run.farm_id.as_str(), agent = run.agent_id.as_str(),
            "Scheduled farm run queued");
        Ok(JobResult::success()
            .with_stats(1, 0)
            .with_message(format!("queued farm run {}", run.id)))
    }
}

#[async_trait::async_trait]
impl Job for FarmScheduledRunJob {
    fn name(&self) -> &'static str {
        "farm_scheduled_run"
    }
    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }
    fn description(&self) -> &'static str {
        "Queues one agent-farm run on the scheduler's behalf (triggered_via = scheduler)"
    }
    fn schedule(&self) -> &'static str {
        "0 0 2 * * *"
    }
    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        Ok(Self::queue(ctx).await?)
    }
}

systemprompt::traits::submit_job!(&FarmScheduledRunJob);
