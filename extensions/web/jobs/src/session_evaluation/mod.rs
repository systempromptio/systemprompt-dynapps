//! Session evaluation: the LLM-as-judge worker.
//!
//! Each tick: enqueue sessions that closed since last time, lease a batch of
//! `(session, criterion)` rows, judge each through the gateway as this job's
//! actor, and record a run whatever the outcome. Nothing here runs inside a
//! hook request; `/hooks/track` only records the events this reads.
//!
//! Gated twice: the governance-owning plugin must set `hooks.evaluation`, and
//! `services/evaluation/config.yaml` must have `enabled: true`. Either off is
//! a successful no-op tick, reported as such.

mod ai_service;

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::config::AppPaths;
use systemprompt::database::DbPool;
use systemprompt::identifiers::Actor;
use systemprompt::loader::ServicesBootstrap;
use systemprompt::models::ServicesConfig;
use systemprompt::system::AppContext;
use systemprompt::traits::{Job, JobContext, JobResult};
use systemprompt_web_admin::evaluation::Judge;
use systemprompt_web_admin::repositories::config::evaluation::{
    CriterionName, LoadedEvaluation, load_evaluation,
};
use systemprompt_web_admin::repositories::dashboard::usage_aggregations::update_session_title;
use systemprompt_web_admin::repositories::evaluation::{queue, runs, runs_summary};

use crate::JobError;

#[derive(Debug, Clone, Copy, Default)]
pub struct SessionEvaluationJob;

pub(crate) const JOB_NAME: &str = "session_evaluation";

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
}

impl SessionEvaluationJob {
    async fn run_tick(tick: &Tick<'_>) -> Result<JobResult, JobError> {
        let close = queue::CloseParams {
            quiet_minutes: tick.config.session_close.quiet_minutes,
            min_user_prompts: tick.config.session_close.min_user_prompts,
            lookback_hours: tick.config.session_close.lookback_hours,
            criteria: &CriterionName::ALL,
        };
        let enqueued = queue::insert_closed_sessions(tick.pool, &close).await?;
        let leases = queue::claim_leases(
            tick.pool,
            tick.config.worker.batch_size,
            tick.config.worker.lease_seconds,
        )
        .await?;
        let mut succeeded = 0u64;
        let mut failed = 0u64;
        for lease in &leases {
            match tick.judge.evaluate(tick.pool, lease).await {
                Ok(verdict) => {
                    runs::insert_run(tick.pool, &verdict.run).await?;
                    if verdict.run.succeeded {
                        succeeded += 1;
                        if let Some(title) = verdict.title {
                            update_session_title(
                                tick.pool,
                                &lease.session_id,
                                &title.title,
                                &title.summary,
                            )
                            .await?;
                        }
                        queue::complete_lease(tick.pool, lease).await?;
                    } else {
                        failed += 1;
                        let error = verdict.run.error.as_deref().unwrap_or("judge failed");
                        tracing::warn!(session = %lease.session_id, criterion = %lease.criterion, error, "Session evaluation failed");
                        queue::retry_lease(
                            tick.pool,
                            lease,
                            error,
                            tick.config.worker.max_attempts,
                        )
                        .await?;
                    }
                },
                Err(error) => {
                    failed += 1;
                    tracing::error!(%error, session = %lease.session_id, criterion = %lease.criterion, "Session evaluation could not run");
                    queue::retry_lease(
                        tick.pool,
                        lease,
                        &error.to_string(),
                        tick.config.worker.max_attempts,
                    )
                    .await?;
                },
            }
        }
        tick.judge.ai.audit_tasks().close();
        tick.judge.ai.audit_tasks().wait().await;
        runs_summary::update_run_costs(tick.pool).await?;
        Ok(JobResult::success()
            .with_stats(succeeded, failed)
            .with_message(format!("{enqueued} enqueued, {} leased", leases.len())))
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

        let judge = Judge {
            ai: ai_service::build_ai_service(db, app, services)?,
            config: Arc::new(config),
            actor: Actor::job(ctx.actor().user_id.clone(), JOB_NAME),
        };
        let pool = db.write_pool();
        let tick = Tick {
            pool: &pool,
            config: Arc::clone(&judge.config),
            judge,
        };
        Ok(Self::run_tick(&tick).await?)
    }
}

systemprompt::traits::submit_job!(&SessionEvaluationJob);
