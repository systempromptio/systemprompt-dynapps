//! Generating an AI report in the moment. `POST /admin/analysis/reports`
//! writes the row with its digest and a lease, then hands it to
//! [`generate_report`] on a background task; the report page polls the row
//! until it is `generated` or `failed`. There is no queue and no scheduler:
//! a report exists because someone asked for it, and it is written by one
//! JSON-answer call to the judge's model, audited under a job actor
//! (`analysis_report`) so the call is never counted as a conversation.

mod prompt;

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::ai::{AiMessage, AiRequest, AiService};
use systemprompt::identifiers::{Actor, AgentName, ContextId, SessionId, TraceId, UserId};
use systemprompt::loader::ConfigLoader;
use systemprompt::models::execution::context::RequestContext;

use crate::repositories::analysis::reports::{
    AnalysisReportRow, ReportCompletion, fail_report, render_digest_text, update_report_completion,
};
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

pub(crate) use prompt::{findings_schema, parse_findings};

const ACTOR_JOB_NAME: &str = "analysis_report";
const AGENT_NAME: &str = "analysis-reporter";
const MAX_OUTPUT_TOKENS: u32 = 4_096;

const JUDGE_JOB_NAME: &str = "conversation_judge";
const REPORTER_CONTEXT_SEED: &[u8] = b"dynapps:analysis_report";

#[derive(Debug, Clone)]
pub(crate) struct ReportModel {
    pub(crate) provider: String,
    pub(crate) model: String,
}

impl ReportModel {
    pub(crate) fn judge_model() -> AdminResult<Self> {
        let services = ConfigLoader::load().map_err(AdminError::internal)?;
        let judge = services
            .scheduler
            .as_ref()
            .and_then(|s| s.jobs.iter().find(|j| j.name == JUDGE_JOB_NAME));
        let provider = judge
            .and_then(|j| j.parameters.get("provider").cloned())
            .filter(|p| !p.is_empty())
            .unwrap_or_else(|| services.ai.default_provider.clone());
        let model = judge
            .and_then(|j| j.parameters.get("model").cloned())
            .filter(|m| !m.is_empty())
            .or_else(|| {
                services
                    .ai
                    .providers
                    .get(&provider)
                    .map(|p| p.default_model.clone())
                    .filter(|m| !m.is_empty())
            })
            .ok_or_else(|| {
                AdminError::internal(format!("no model is configured for provider {provider}"))
            })?;
        Ok(Self { provider, model })
    }
}

fn request_context(actor: Actor) -> AdminResult<RequestContext> {
    Ok(RequestContext::new(
        SessionId::generate(),
        TraceId::new(uuid::Uuid::new_v4().to_string()),
        ContextId::from_uuid(uuid::Uuid::new_v5(
            &uuid::Uuid::NAMESPACE_OID,
            REPORTER_CONTEXT_SEED,
        )),
        AgentName::try_new(AGENT_NAME).map_err(AdminError::internal)?,
        actor,
    ))
}

fn digest_text(row: &AnalysisReportRow) -> String {
    let digest = render_digest_text(&row.inputs.digest);
    let scope = row.inputs.filter_label.as_deref().map_or_else(
        || {
            row.scope_id
                .as_deref()
                .map(|id| format!(" {id}"))
                .unwrap_or_default()
        },
        |label| format!(" — {label}"),
    );
    format!("SCOPE {}{scope}\n{digest}", row.scope_kind)
}

struct Grant<'a> {
    requested_by: &'a UserId,
    lease: &'a str,
}

async fn call_model(
    pool: &PgPool,
    ai: &AiService,
    grant: Grant<'_>,
    row: &AnalysisReportRow,
    model: &ReportModel,
) -> AdminResult<()> {
    let request = AiRequest::builder(
        vec![
            AiMessage::system(format!(
                "{}\nReply with one JSON object and nothing else, matching this schema:\n{}",
                prompt::SYSTEM_PROMPT,
                findings_schema()
            )),
            AiMessage::user(digest_text(row)),
        ],
        model.provider.as_str(),
        model.model.as_str(),
        MAX_OUTPUT_TOKENS,
        request_context(Actor::job(grant.requested_by.clone(), ACTOR_JOB_NAME))?,
    )
    .build();
    let response = ai.generate(&request).await.map_err(AdminError::internal)?;
    let findings = parse_findings(&response.content).map_err(AdminError::internal)?;
    let request_id = response.request_id.to_string();
    let written = update_report_completion(
        pool,
        ReportCompletion {
            id: &row.id,
            lease_token: grant.lease,
            findings: &findings,
            provider: &model.provider,
            model: &model.model,
            ai_request_id: Some(request_id.as_str()),
            input_tokens: response.input_tokens.and_then(|t| i32::try_from(t).ok()),
            output_tokens: response.output_tokens.and_then(|t| i32::try_from(t).ok()),
        },
    )
    .await?;
    if !written {
        tracing::warn!(report_id = %row.id, "analysis report lease was lost before completion");
    }
    Ok(())
}

pub(crate) async fn generate_report(
    pool: Arc<PgPool>,
    ai: Option<Arc<AiService>>,
    requested_by: UserId,
    row: AnalysisReportRow,
    lease: String,
) {
    let outcome = match ai {
        None => Err(AdminError::internal(
            "no AI provider is configured on this instance, so a report cannot be written",
        )),
        Some(ai) => match ReportModel::judge_model() {
            Err(error) => Err(error),
            Ok(model) => {
                let grant = Grant {
                    requested_by: &requested_by,
                    lease: &lease,
                };
                call_model(&pool, &ai, grant, &row, &model).await
            },
        },
    };
    if let Err(error) = outcome {
        tracing::warn!(report_id = %row.id, %error, "analysis report failed");
        if let Err(write) = fail_report(&pool, &row.id, &lease, &error.to_string()).await {
            tracing::error!(report_id = %row.id, %write, "analysis report failure could not be recorded");
        }
    }
}
