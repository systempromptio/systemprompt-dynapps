//! The report writer: one leased scope, a digest of the judge's figures, one
//! model call made as the job actor, and a generated row whatever happens.
//!
//! A failed call or an unparseable answer is a retry on the lease, not a
//! generated row: a report is either the model's findings or nothing.

pub mod inputs;
pub mod prompt;
pub mod schema;

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::ai::{AiMessage, AiRequest, AiService, StructuredOutputOptions};
use systemprompt::identifiers::{Actor, AgentName, ContextId, SessionId, TraceId, UserId};
use systemprompt::models::auth::UserType;
use systemprompt::models::execution::context::RequestContext;

use crate::repositories::evaluation::LoadedEvaluation;
use crate::repositories::evaluation::report_inputs::ReportWindow;
use crate::repositories::evaluation::reports::{
    GeneratedReport, ReportLease, update_report_costs, update_report_generated, update_report_retry,
};

pub use schema::{ReportFindings, parse_findings, report_findings_schema};

pub const REPORT_AGENT_NAME: &str = "evaluation-report";
pub const JOB_NAME: &str = "evaluation_report";
const JOB_CONTEXT_SEED: &[u8] = b"astound:evaluation_report";

#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("report request context: {0}")]
    Context(#[from] systemprompt::identifiers::error::IdValidationError),
}

#[derive(Debug, thiserror::Error)]
enum ReportFailure {
    #[error("gateway refused the report call: {0}")]
    Gateway(#[from] systemprompt::ai::error::AiError),
    #[error("report was not valid JSON for the schema: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("nothing judged in the window")]
    Empty,
}

#[derive(Debug)]
pub struct Reporter {
    pub ai: Arc<AiService>,
    pub config: Arc<LoadedEvaluation>,
    pub actor: Actor,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ReportPass {
    pub leased: usize,
    pub generated: u64,
    pub failed: u64,
}

impl Reporter {
    // Why: every report call is made as the job actor, so the spend is
    // filed under `actor_kind = job` and never against the admin who clicked.
    #[must_use]
    pub fn from_parts(ai: Arc<AiService>, config: Arc<LoadedEvaluation>, owner: UserId) -> Self {
        Self {
            ai,
            config,
            actor: Actor::job(owner, JOB_NAME),
        }
    }

    // Why: an empty session id keeps core from binding the call to a user
    // session; the fixed context id keeps it out of every conversation.
    fn request_context(&self) -> Result<RequestContext, ReportError> {
        Ok(RequestContext::new(
            SessionId::new(""),
            TraceId::new(uuid::Uuid::new_v4().to_string()),
            ContextId::from_uuid(uuid::Uuid::new_v5(
                &uuid::Uuid::NAMESPACE_OID,
                JOB_CONTEXT_SEED,
            )),
            AgentName::try_new(REPORT_AGENT_NAME)?,
        )
        .with_actor(self.actor.clone())
        .with_user_type(UserType::Service))
    }

    async fn generate(
        &self,
        pool: &PgPool,
        lease: &ReportLease,
    ) -> Result<Result<(), ReportFailure>, ReportError> {
        let report = &self.config.report;
        let window = ReportWindow {
            start: lease.window_start,
            end: lease.window_end,
        };
        let assembled = inputs::assemble_digest(
            pool,
            &lease.scope,
            window,
            usize::try_from(report.input_token_budget).unwrap_or(20_000),
        )
        .await?;
        if assembled.digest.totals.scored_sessions == 0 {
            return Ok(Err(ReportFailure::Empty));
        }
        let request = AiRequest::builder(
            vec![AiMessage::user(prompt::build_user_prompt(&assembled.json))],
            report.provider.clone(),
            report.model.clone(),
            report.max_output_tokens,
            self.request_context()?,
        )
        .with_system_prompt(prompt::build_system_prompt(&report.instructions_markdown))
        .with_structured_output(StructuredOutputOptions::with_schema(
            report_findings_schema(),
        ))
        .build();
        let response = match self.ai.generate(&request).await {
            Ok(response) => response,
            Err(error) => return Ok(Err(ReportFailure::Gateway(error))),
        };
        let findings = match parse_findings(&response.content, &assembled.known) {
            Ok(findings) => findings,
            Err(error) => return Ok(Err(ReportFailure::InvalidJson(error))),
        };
        // Why: discard-ok: an in-memory struct always serialises
        let inputs = serde_json::to_value(&assembled.digest).unwrap_or_default();
        // Why: discard-ok: an in-memory struct always serialises
        let findings = serde_json::to_value(&findings).unwrap_or_default();
        update_report_generated(
            pool,
            lease,
            GeneratedReport {
                provider: &report.provider,
                model: &report.model,
                ai_request_id: Some(&response.request_id.to_string()),
                input_tokens: response.input_tokens.map(i64::from),
                output_tokens: response.output_tokens.map(i64::from),
                latency_ms: i32::try_from(response.latency_ms).ok(),
                inputs: &inputs,
                findings: &findings,
                source_run_count: assembled.digest.totals.runs,
                source_last_run_at: assembled.digest.totals.last_run_at,
            },
        )
        .await?;
        Ok(Ok(()))
    }
}

// Why: writes every leased scope; a lease that fails is retried with backoff.
pub async fn generate_pending(
    pool: &PgPool,
    reporter: &Reporter,
    leases: &[ReportLease],
) -> Result<ReportPass, ReportError> {
    let mut pass = ReportPass {
        leased: leases.len(),
        ..ReportPass::default()
    };
    for lease in leases {
        match reporter.generate(pool, lease).await {
            Ok(Ok(())) => pass.generated += 1,
            Ok(Err(failure)) => {
                pass.failed += 1;
                tracing::warn!(scope = ?lease.scope, %failure, attempt = lease.attempts, "Evaluation report not written");
                update_report_retry(pool, lease, &failure.to_string()).await?;
            },
            Err(error) => {
                pass.failed += 1;
                tracing::error!(scope = ?lease.scope, %error, "Evaluation report could not run");
                update_report_retry(pool, lease, &error.to_string()).await?;
            },
        }
    }
    update_report_costs(pool).await?;
    Ok(pass)
}
