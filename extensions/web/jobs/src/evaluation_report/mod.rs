//! The evaluation report writer: the zoomed-out view on
//! /admin/analysis/evaluation, one report per scope (the instance and each
//! marketplace), regenerated once enough new judge runs have landed.
//!
//! Each tick: when the profile says evaluation is automatic, queue every
//! scope the staleness rule considers due; then lease what is pending — a
//! console click leaves a pending row behind whether or not the profile is
//! automatic — assemble the judge's figures for the scope and ask the report
//! model for themes and actions. Gated like `session_evaluation`: the
//! governance-owning plugin's `hooks.evaluation`, `services/evaluation/
//! config.yaml`'s `enabled`, and an `AiService` in the process. A daily cost
//! cap stops the run rather than letting a backlog burn budget.
//!
//! Force one scope regardless of staleness:
//! `systemprompt infra jobs run evaluation_report -p scope=global` or
//! `-p scope=marketplace:<id>`.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::config::{AppPaths, ProfileBootstrap};
use systemprompt::database::DbPool;
use systemprompt::loader::ServicesBootstrap;
use systemprompt::system::AppContext;
use systemprompt::traits::{Job, JobContext, JobResult};
use systemprompt_web_admin::evaluation::report::{Reporter, generate_pending};
use systemprompt_web_admin::repositories::config::evaluation::load_evaluation;
use systemprompt_web_admin::repositories::evaluation::reports::{
    ReportScope, Staleness, claim_report_leases, get_report_spend_today, insert_manual_report,
    insert_stale_scopes,
};

use crate::JobError;
use crate::session_evaluation::evaluation_owner;

#[derive(Debug, Clone, Copy, Default)]
pub struct EvaluationReportJob;

pub(crate) const JOB_NAME: &str = systemprompt_web_admin::evaluation::report::JOB_NAME;
const BATCH: i64 = 4;

struct Params {
    daily_cost_cap_microdollars: i64,
    scope: Option<ReportScope>,
}

impl Params {
    fn from_context(ctx: &JobContext) -> Result<Self, JobError> {
        let scope = match ctx.get_parameter_parsed::<String>("scope")? {
            None => None,
            Some(raw) => Some(parse_scope(&raw)?),
        };
        Ok(Self {
            daily_cost_cap_microdollars: ctx
                .get_parameter_parsed::<i64>("daily_cost_cap_microdollars")?
                .unwrap_or(1_000_000),
            scope,
        })
    }
}

fn parse_scope(raw: &str) -> Result<ReportScope, JobError> {
    match raw.trim().split_once(':') {
        None if raw.trim() == "global" => Ok(ReportScope::Global),
        Some(("marketplace", id)) if !id.is_empty() => Ok(ReportScope::Marketplace(id.to_owned())),
        _ => Err(JobError::other(
            "scope must be `global` or `marketplace:<id>`",
        )),
    }
}

struct Tick<'a> {
    pool: &'a PgPool,
    reporter: Reporter,
    params: Params,
    automatic: bool,
}

impl EvaluationReportJob {
    async fn run_tick(tick: &Tick<'_>) -> Result<JobResult, JobError> {
        let report = &tick.reporter.config.report;
        let window_days = i32::try_from(report.window_days).unwrap_or(7);
        let queued = if let Some(scope) = &tick.params.scope {
            insert_manual_report(tick.pool, scope, window_days, &tick.reporter.actor.user_id)
                .await?;
            1
        } else if tick.automatic {
            insert_stale_scopes(
                tick.pool,
                Staleness {
                    window_days,
                    min_new_runs: i64::from(report.min_new_runs),
                    max_age_hours: i32::try_from(report.max_age_hours).unwrap_or(24),
                },
            )
            .await?
        } else {
            0
        };
        let spend = get_report_spend_today(tick.pool, JOB_NAME).await?;
        if spend >= tick.params.daily_cost_cap_microdollars {
            return Ok(JobResult::success().with_message(format!(
                "daily cost cap reached ({spend} of {} microdollars); {queued} queued, none written",
                tick.params.daily_cost_cap_microdollars
            )));
        }
        let leases = claim_report_leases(tick.pool, BATCH, tick.params.scope.as_ref()).await?;
        let pass = generate_pending(tick.pool, &tick.reporter, &leases).await?;
        Ok(JobResult::success()
            .with_stats(pass.generated, pass.failed)
            .with_message(format!(
                "automatic={}, {queued} queued, {} leased",
                tick.automatic, pass.leased
            )))
    }
}

#[async_trait::async_trait]
impl Job for EvaluationReportJob {
    fn name(&self) -> &'static str {
        JOB_NAME
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn description(&self) -> &'static str {
        "Writes the evaluation report per scope from the judge's scores once enough new runs have landed"
    }

    fn schedule(&self) -> &'static str {
        "0 15 * * * *"
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
        let Some(ai) = app.ai_service_arc() else {
            return Ok(
                JobResult::success().with_message("no AI provider is configured; report idle")
            );
        };
        let params = Params::from_context(ctx)?;
        let automatic = ProfileBootstrap::get()
            .map_err(JobError::from)?
            .evaluation
            .automatic;
        let reporter = Reporter::from_parts(ai, Arc::new(config), ctx.actor().user_id.clone());
        let pool = db.write_pool();
        let tick = Tick {
            pool: &pool,
            reporter,
            params,
            automatic,
        };
        Ok(Self::run_tick(&tick).await?)
    }
}

systemprompt::traits::submit_job!(&EvaluationReportJob);
