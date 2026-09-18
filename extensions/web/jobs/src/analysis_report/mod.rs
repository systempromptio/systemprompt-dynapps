//! `analysis_report` job: the on-demand AI report over the deterministic
//! record.
//!
//! Manual only — `schedulable()` is false, so the scheduler never ticks it.
//! Rows arrive from the console (`POST /admin/analysis/reports`, or "Report
//! on this view" on Conversations and Skills), each carrying the digest that
//! was computed for its scope and window at request time. A run leases a few
//! pending rows, hands each digest — figures only, never a transcript — to
//! the configured model in one structured-output call, and stores the
//! headline, assessment, themes with evidence links and recommendations on
//! the row with the audit id, tokens and cost of that call. Run it with
//! `systemprompt infra jobs run analysis_report`.
//!
//! Report calls go through the process's shared `AiService` as this job's
//! actor under their own context id, so they are audited like any request and
//! never counted as a conversation.

mod prompt;

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::ai::{AiMessage, AiRequest, AiService, StructuredOutputOptions};
use systemprompt::database::DbPool;
use systemprompt::identifiers::{Actor, AgentName, ContextId, SessionId, TraceId};
use systemprompt::models::execution::context::RequestContext;
use systemprompt::system::AppContext;
use systemprompt::traits::{Job, JobContext, JobResult};
use systemprompt_web_admin::repositories::analysis::reports::{
    AnalysisReportRow, ReportCompletion, claim_pending_reports, fail_report, render_digest_text,
    update_report_completion,
};

use crate::JobError;

pub use prompt::{findings_schema, parse_findings};

pub(crate) const JOB_NAME: &str = "analysis_report";
const AGENT_NAME: &str = "analysis-reporter";
const LEASE_SECONDS: i64 = 600;

// Why: the reporter's own requests need a context of their own, distinct
// from the judge's, so each job's spend can be told apart in the audit.
const JOB_CONTEXT_SEED: &[u8] = b"astound:analysis_report";

// Why: tunables, each overridable from `systemprompt infra jobs run
// analysis_report -p key=value`.
#[derive(Debug, Clone)]
struct ReportParams {
    provider: String,
    model: String,
    batch_size: i64,
    max_output_tokens: u32,
}

impl ReportParams {
    fn from_context(ctx: &JobContext) -> Result<Self, JobError> {
        let batch = ctx.get_parameter_parsed::<i64>("batch_size")?.unwrap_or(3);
        let max_out = ctx
            .get_parameter_parsed::<i64>("max_output_tokens")?
            .unwrap_or(4_096);
        Ok(Self {
            provider: ctx
                .get_parameter_parsed::<String>("provider")?
                .unwrap_or_else(|| "gemini".to_owned()),
            model: ctx
                .get_parameter_parsed::<String>("model")?
                .unwrap_or_else(|| "gemini-3.8-flash".to_owned()),
            batch_size: batch.clamp(1, 10),
            max_output_tokens: u32::try_from(max_out.clamp(512, 65_536)).unwrap_or(4_096),
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AnalysisReportJob;

struct Reporter {
    ai: Arc<AiService>,
    actor: Actor,
    params: ReportParams,
}

impl Reporter {
    // Why: an empty session id keeps core from binding the call to a user
    // session; the job context id keeps it out of every conversation.
    fn request_context(&self) -> Result<RequestContext, JobError> {
        Ok(RequestContext::new(
            SessionId::new(""),
            TraceId::new(uuid::Uuid::new_v4().to_string()),
            ContextId::from_uuid(uuid::Uuid::new_v5(
                &uuid::Uuid::NAMESPACE_OID,
                JOB_CONTEXT_SEED,
            )),
            AgentName::try_new(AGENT_NAME)?,
        )
        .with_actor(self.actor.clone()))
    }

    async fn generate(&self, pool: &PgPool, row: &AnalysisReportRow) -> Result<(), JobError> {
        let Some(lease) = row.lease_token.as_deref() else {
            return Err(JobError::other("report row was returned without a lease"));
        };
        let mut digest = render_digest_text(&row.inputs.digest);
        if let Some(label) = row.inputs.filter_label.as_deref() {
            digest = format!("SCOPE {} — {label}\n{digest}", row.scope_kind);
        } else {
            digest = format!(
                "SCOPE {}{}\n{digest}",
                row.scope_kind,
                row.scope_id
                    .as_deref()
                    .map(|id| format!(" {id}"))
                    .unwrap_or_default()
            );
        }
        let request = AiRequest::builder(
            vec![
                AiMessage::system(prompt::SYSTEM_PROMPT),
                AiMessage::user(digest),
            ],
            self.params.provider.as_str(),
            self.params.model.as_str(),
            self.params.max_output_tokens,
            self.request_context()?,
        )
        .with_structured_output(StructuredOutputOptions::with_schema(findings_schema()))
        .build();
        let response = self.ai.generate(&request).await?;
        let findings = parse_findings(&response.content)?;
        let request_id = response.request_id.to_string();
        let written = update_report_completion(
            pool,
            ReportCompletion {
                id: &row.id,
                lease_token: lease,
                findings: &findings,
                provider: &self.params.provider,
                model: &self.params.model,
                ai_request_id: Some(request_id.as_str()),
                input_tokens: response.input_tokens.and_then(|t| i32::try_from(t).ok()),
                output_tokens: response.output_tokens.and_then(|t| i32::try_from(t).ok()),
            },
        )
        .await?;
        if !written {
            tracing::warn!(report_id = %row.id, "analysis report lease expired before completion");
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl Job for AnalysisReportJob {
    fn name(&self) -> &'static str {
        JOB_NAME
    }

    fn description(&self) -> &'static str {
        "Writes the on-demand AI report over the conversation record for each request queued from the console"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    // Why: a valid expression the scheduler never uses — `schedulable` is
    // false, so reports are generated only when someone runs the job.
    fn schedule(&self) -> &'static str {
        "0 0 3 * * *"
    }

    fn schedulable(&self) -> bool {
        false
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
        let params = ReportParams::from_context(ctx)?;
        let Some(ai) = app.ai_service_arc() else {
            return Ok(JobResult::success()
                .with_message("no AI provider is configured; reports stay queued"));
        };
        let reporter = Reporter {
            ai,
            actor: Actor::job(ctx.actor().user_id.clone(), JOB_NAME),
            params,
        };
        let pool = db.write_pool();
        let rows = claim_pending_reports(&pool, reporter.params.batch_size, LEASE_SECONDS)
            .await
            .map_err(JobError::from)?;
        let (mut generated, mut failed) = (0u64, 0u64);
        for row in &rows {
            match reporter.generate(&pool, row).await {
                Ok(()) => generated += 1,
                Err(error) => {
                    failed += 1;
                    tracing::warn!(%error, report_id = %row.id, "analysis report failed; will retry");
                    fail_report(
                        &pool,
                        &row.id,
                        row.lease_token.as_deref().unwrap_or_default(),
                        &error.to_string(),
                    )
                    .await
                    .map_err(JobError::from)?;
                },
            }
        }
        Ok(JobResult::success()
            .with_stats(generated, failed)
            .with_message(format!("{generated} report(s) generated, {failed} failed")))
    }
}

systemprompt::traits::submit_job!(&AnalysisReportJob);
