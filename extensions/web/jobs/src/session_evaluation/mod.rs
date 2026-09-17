//! Session evaluation: the LLM-as-judge worker.
//!
//! Each tick: enqueue sessions that closed since last time (only when the
//! profile says scoring is automatic), lease a batch of `(session,
//! criterion)` rows, judge each through the process's gateway client as this
//! job's actor, and record a run whatever the outcome. Nothing here runs
//! inside a hook request; `/hooks/track` only records the events this reads.
//!
//! Gated twice: the governance-owning plugin must set `hooks.evaluation`, and
//! `services/evaluation/config.yaml` must have `enabled: true`. Either off is
//! a successful no-op tick, reported as such. With `evaluation.automatic`
//! off in the profile the tick still drains — a console click that died
//! mid-run leaves leased rows behind, and this is what finishes them.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::config::{AppPaths, ProfileBootstrap};
use systemprompt::database::DbPool;
use systemprompt::loader::ServicesBootstrap;
use systemprompt::models::ServicesConfig;
use systemprompt::system::AppContext;
use systemprompt::traits::{Job, JobContext, JobResult};
use systemprompt_web_admin::evaluation::{Judge, Trigger, drain};
use systemprompt_web_admin::repositories::config::evaluation::{
    CriterionName, LoadedEvaluation, load_evaluation,
};
use systemprompt_web_admin::repositories::evaluation::queue;

use crate::JobError;

#[derive(Debug, Clone, Copy, Default)]
pub struct SessionEvaluationJob;

pub(crate) const JOB_NAME: &str = systemprompt_web_admin::evaluation::JOB_NAME;

// Why: the plugin that switches the engine on, if any.
#[must_use]
pub(crate) fn evaluation_owner(services: &ServicesConfig) -> Option<&str> {
    services
        .plugins
        .values()
        .find(|p| p.enabled && p.hooks.evaluation)
        .map(|p| p.id.as_str())
}

struct Tick<'a> {
    pool: &'a PgPool,
    config: Arc<LoadedEvaluation>,
    judge: Judge,
    automatic: bool,
}

impl SessionEvaluationJob {
    async fn run_tick(tick: &Tick<'_>) -> Result<JobResult, JobError> {
        let enqueued = if tick.automatic {
            let close = queue::CloseParams {
                quiet_minutes: tick.config.session_close.quiet_minutes,
                min_user_prompts: tick.config.session_close.min_user_prompts,
                lookback_hours: tick.config.session_close.lookback_hours,
                criteria: &CriterionName::ALL,
            };
            queue::insert_closed_sessions(tick.pool, &close).await?
        } else {
            0
        };
        let leases = queue::claim_leases(
            tick.pool,
            tick.config.worker.batch_size,
            tick.config.worker.lease_seconds,
        )
        .await?;
        let report = drain(tick.pool, &tick.judge, &leases, &Trigger::Automatic).await?;
        Ok(JobResult::success()
            .with_stats(report.succeeded, report.failed)
            .with_message(format!(
                "automatic={}, {enqueued} enqueued, {} leased",
                tick.automatic, report.leased
            )))
    }
}

#[async_trait::async_trait]
impl Job for SessionEvaluationJob {
    fn name(&self) -> &'static str {
        JOB_NAME
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn description(&self) -> &'static str {
        "Scores closed sessions per criterion through the configured judge models"
    }

    fn schedule(&self) -> &'static str {
        "0 * * * * *"
    }

    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        let db = ctx
            .db_pool::<DbPool>()
            .ok_or(JobError::MissingContext("DbPool"))?;
        let app = ctx
            .app_context::<Arc<AppContext>>()
            .ok_or(JobError::MissingContext("AppContext"))?;
        let paths = ctx
            .app_paths::<Arc<AppPaths>>()
            .ok_or(JobError::MissingContext("AppPaths"))?;
        let services = ServicesBootstrap::get().map_err(JobError::from)?;

        let Some(owner) = evaluation_owner(services) else {
            return Ok(JobResult::success().with_message("no plugin sets hooks.evaluation"));
        };
        let config = load_evaluation(paths.system().services(), &services.providers, &services.ai)
            .map_err(JobError::from)?
            .ok_or_else(|| {
                JobError::Config(format!(
                    "plugin '{owner}' sets hooks.evaluation but services/evaluation/config.yaml is missing"
                ))
            })?;
        if !config.enabled {
            return Ok(JobResult::success().with_message("evaluation disabled in config"));
        }
        let ai = app.ai_service_arc().ok_or_else(|| {
            JobError::Config("inference is not configured on this instance".to_owned())
        })?;
        let automatic = ProfileBootstrap::get()
            .map_err(JobError::from)?
            .evaluation
            .automatic;

        let judge = Judge::from_parts(ai, Arc::new(config), ctx.actor().user_id.clone());
        let pool = db.write_pool();
        let tick = Tick {
            pool: &pool,
            config: Arc::clone(&judge.config),
            judge,
            automatic,
        };
        Ok(Self::run_tick(&tick).await?)
    }
}

systemprompt::traits::submit_job!(&SessionEvaluationJob);
