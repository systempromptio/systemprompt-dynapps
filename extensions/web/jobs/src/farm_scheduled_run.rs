//! `farm_scheduled_run` job: queues one agent-farm run on core's
//! `agent_runs` on the scheduler's behalf.
//!
//! The scheduler entry names the farm, the agent and the person the run
//! launches as (`farm`, `agent`, `launcher` parameters), and optionally the
//! eval `case` the run is launched as, so it is judged when it finishes. The
//! run is recorded with `triggered_via = scheduler`; the farm's runner service
//! claims it like any other queued run. The job only queues: it never waits for
//! the run. Where the launcher is not an active person (every environment but
//! production) the tick logs that and queues nothing.

use std::sync::Arc;

use systemprompt::config::AppPaths;
use systemprompt::database::DbPool;
use systemprompt::system::AppContext;
use systemprompt::traits::{Job, JobContext, JobResult};
use systemprompt_web_admin_farm::services::runner::fly::registry::RemoteRuns;
use systemprompt_web_admin_farm::services::scheduled::{ScheduledLaunch, launch_scheduled};
use systemprompt_web_admin_farm::state::{FarmHost, build_farm_state_with_ai};
use systemprompt_web_admin_farm::types::{FarmAgentId, FarmId};

use crate::error::JobError;

#[derive(Debug, Clone, Copy, Default)]
pub struct FarmScheduledRunJob;

// Why: this job only queues the run on core's `agent_runs`; the runner service
// that claims it owns the shared remote-run registry.
fn queue_only_runs() -> Arc<RemoteRuns> {
    Arc::new(RemoteRuns::default())
}

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
            case: ctx
                .get_parameter_parsed::<String>("case")?
                .map(|case| case.trim().to_owned())
                .filter(|case| !case.is_empty()),
        };
        let db = ctx.get::<DbPool>()?;
        let paths = ctx.get::<Arc<AppPaths>>().ok().map(Arc::clone);
        let app = ctx.get::<Arc<AppContext>>()?;
        let host = FarmHost {
            artefacts: Arc::clone(app.file_storage()),
            instance: app.config().instance_id.clone(),
            runs: queue_only_runs(),
            tasks: app.background_tasks().clone(),
            services:
                systemprompt_web_admin_core::extension_services::ExtensionServices::from_context(
                    app.as_ref(),
                ),
        };
        let state = build_farm_state_with_ai(
            db.write_pool(),
            paths.as_deref(),
            app.ai_service_arc(),
            host,
        );
        let launcher = scheduled.launcher.clone();
        let Some(run) = launch_scheduled(&state, scheduled).await? else {
            tracing::info!(launcher = %launcher, "Scheduled farm run skipped: launcher is not an active person here");
            return Ok(JobResult::success().with_stats(0, 0).with_message(format!(
                "skipped: launcher {launcher} is not an active person"
            )));
        };
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
