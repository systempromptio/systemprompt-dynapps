//! The judge: one criterion of one closed session, scored by a model call
//! made as the job actor, recorded whatever happens.
//!
//! A failed call still produces a run row with `status = failed` and the
//! error, so the dashboard shows the attempt and the queue's backoff decides
//! whether it is tried again.

pub mod prompt;
pub mod report;
pub mod schema;
pub mod worker;

use std::sync::Arc;

use systemprompt::ai::{AiMessage, AiRequest, AiService, StructuredOutputOptions};
use systemprompt::identifiers::{Actor, AgentName, ContextId, SessionId, TraceId, UserId};
use systemprompt::models::auth::UserType;
use systemprompt::models::execution::context::RequestContext;

use crate::repositories::evaluation::inputs::{AssembledInputs, assemble_inputs};
use crate::repositories::evaluation::metrics::SessionMetrics;
use crate::repositories::evaluation::queue::EvaluationLease;
use crate::repositories::evaluation::runs::{NewEvaluationRun, RunTrigger};
use crate::repositories::evaluation::scoring::{cost_score, speed_score};
use crate::repositories::evaluation::{
    LoadedCriterion, LoadedEvaluation, MetricThresholds, Scoring,
};

pub use schema::{JudgeOutput, judge_output_schema};
pub use worker::{DrainReport, Trigger, drain};

pub const JUDGE_AGENT_NAME: &str = "session-evaluation";
pub const JOB_NAME: &str = "session_evaluation";

#[derive(Debug, thiserror::Error)]
pub enum JudgeError {
    #[error("criterion '{0}' is not configured")]
    UnknownCriterion(String),
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("judge request context: {0}")]
    Context(#[from] systemprompt::identifiers::error::IdValidationError),
}

/// Why one judge call produced no usable verdict; recorded on the failed run.
#[derive(Debug, thiserror::Error)]
pub enum JudgeFailure {
    #[error("gateway refused the judge call: {0}")]
    Gateway(#[from] systemprompt::ai::error::AiError),
    #[error("judge returned invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("judge returned no score")]
    NoScore,
}

#[derive(Debug)]
pub struct Judge {
    pub ai: Arc<AiService>,
    pub config: Arc<LoadedEvaluation>,
    pub actor: Actor,
}

// Why: what the intent judge learned about the session, for the session row.
#[derive(Debug, Clone)]
pub struct SessionTitle {
    pub title: String,
    pub summary: String,
}

#[derive(Debug)]
pub struct Verdict {
    pub run: NewEvaluationRun,
    pub title: Option<SessionTitle>,
}

impl Judge {
    // Why: every judge call, however it was triggered, is made as the job
    // actor owned by the plugin that switched evaluation on, so the spend is
    // filed under `actor_kind = job` and never against a person.
    #[must_use]
    pub fn from_parts(ai: Arc<AiService>, config: Arc<LoadedEvaluation>, owner: UserId) -> Self {
        Self {
            ai,
            config,
            actor: Actor::job(owner, JOB_NAME),
        }
    }

    // Why: errors here are infrastructure (DB, identifiers); a model failure
    // is returned as a failed run, not an error.
    pub async fn evaluate(
        &self,
        pool: &sqlx::PgPool,
        lease: &EvaluationLease,
    ) -> Result<Verdict, JudgeError> {
        let criterion = self
            .config
            .criterion(lease.criterion)
            .ok_or_else(|| JudgeError::UnknownCriterion(lease.criterion.to_string()))?;
        let inputs =
            assemble_inputs(pool, &lease.user_id, &lease.session_id, &criterion.inputs).await?;
        let mut run = self.blank_run(lease, criterion, &inputs);

        let context = self.request_context(&lease.session_id)?;
        let request = AiRequest::builder(
            vec![AiMessage::user(&run.user_prompt)],
            criterion.provider.clone(),
            criterion.model.clone(),
            criterion.max_output_tokens,
            context,
        )
        .with_system_prompt(run.system_prompt.clone())
        .with_structured_output(StructuredOutputOptions::with_schema(judge_output_schema(
            criterion.name,
            criterion.scoring,
        )))
        .build();

        let response = match self.ai.generate(&request).await {
            Ok(response) => response,
            Err(error) => {
                run.error = Some(JudgeFailure::from(error).to_string());
                return Ok(Verdict { run, title: None });
            },
        };
        run.ai_request_id = Some(response.request_id.to_string());
        run.input_tokens = response.input_tokens.map(i64::from);
        run.output_tokens = response.output_tokens.map(i64::from);
        run.latency_ms = i32::try_from(response.latency_ms).ok();
        run.response = Some(response.content.clone());

        let title = match apply_output(&mut run, criterion.scoring, &response.content) {
            Ok(title) => title,
            Err(error) => {
                run.error = Some(error.to_string());
                return Ok(Verdict { run, title: None });
            },
        };
        Ok(Verdict { run, title })
    }

    // Why: the row as it stands before the model answers — a failed call
    // still records exactly what it was shown.
    fn blank_run(
        &self,
        lease: &EvaluationLease,
        criterion: &LoadedCriterion,
        inputs: &AssembledInputs,
    ) -> NewEvaluationRun {
        let fixed_score = fixed_score(criterion, inputs);
        NewEvaluationRun {
            session_id: lease.session_id.clone(),
            user_id: lease.user_id.clone(),
            host: lease.host.clone(),
            criterion: lease.criterion,
            attempt: lease.attempts,
            succeeded: false,
            score_source: criterion.scoring,
            score: fixed_score.map(i16::from),
            rationale: None,
            evidence: serde_json::json!([]),
            metrics: inputs
                .metrics
                .as_ref()
                .map_or_else(|| serde_json::json!({}), metrics_json),
            provider: criterion.provider.clone(),
            model: criterion.model.clone(),
            ai_request_id: None,
            input_tokens: None,
            output_tokens: None,
            cost_microdollars: None,
            latency_ms: None,
            system_prompt: prompt::build_system_prompt(criterion),
            user_prompt: prompt::build_user_prompt(inputs, fixed_score),
            response: None,
            error: None,
            config_hash: self.config.config_hash.clone(),
            trigger: RunTrigger::Automatic,
            requested_by: None,
        }
    }

    // Why: the judged session is not a `user_sessions` row, and the request's
    // `session_id` column is a foreign key to one, so the judge's own context
    // carries no session. The evaluated session survives in the derived
    // context id and on the run row.
    fn request_context(&self, session_id: &SessionId) -> Result<RequestContext, JudgeError> {
        Ok(RequestContext::new(
            SessionId::new(""),
            TraceId::new(uuid::Uuid::new_v4().to_string()),
            ContextId::derived_from_session(session_id),
            AgentName::try_new(JUDGE_AGENT_NAME)?,
        )
        .with_actor(self.actor.clone())
        .with_user_type(UserType::Service))
    }
}

// Why: parsing and scoring are separated from the request so the run row is
// filled in exactly one place whichever way the model answered.
fn apply_output(
    run: &mut NewEvaluationRun,
    scoring: Scoring,
    content: &str,
) -> Result<Option<SessionTitle>, JudgeFailure> {
    let output: JudgeOutput = serde_json::from_str(content)?;
    if scoring == Scoring::Judge {
        let score = output.score.ok_or(JudgeFailure::NoScore)?;
        run.score = Some(score.clamp(0, 100) as i16);
    }
    run.rationale = Some(output.rationale.clone());
    // Why: discard-ok: an in-memory struct always serialises
    run.evidence = serde_json::to_value(&output.evidence).unwrap_or_default();
    run.succeeded = true;
    Ok(output
        .title
        .filter(|t| !t.trim().is_empty())
        .map(|title| SessionTitle {
            title,
            summary: output.rationale,
        }))
}

// JSON: the metrics snapshot is stored on the run as the judge saw it and
// read back only for display.
fn metrics_json(metrics: &SessionMetrics) -> serde_json::Value {
    // Why: discard-ok: an in-memory struct always serialises
    serde_json::to_value(metrics).unwrap_or_default()
}

fn fixed_score(criterion: &LoadedCriterion, inputs: &AssembledInputs) -> Option<u8> {
    let metrics = inputs.metrics.as_ref()?;
    match criterion.metric? {
        MetricThresholds::Speed {
            target_minutes_per_prompt,
            target_latency_ms,
        } => Some(speed_score(
            metrics,
            target_minutes_per_prompt,
            target_latency_ms,
        )),
        MetricThresholds::Cost {
            budget_microdollars_per_prompt,
            expected_cache_read_ratio,
        } => Some(cost_score(
            metrics,
            budget_microdollars_per_prompt,
            expected_cache_read_ratio,
        )),
    }
}
