//! Launching one paired comparison inside a campaign: the baseline and the
//! candidate are registered as frozen workspaces, development cases only, and
//! every setup failure is retained as an operator-facing diagnostic.

use super::require_admin;
use crate::error::{AdminError, AdminHtmlResult};
use crate::routes::evaluation_state::EvaluationState;
use crate::types::UserContext;
use axum::Extension;
use axum::extract::{Form, Path};
use axum::http::HeaderMap;
use axum::response::Redirect;
use serde::Deserialize;
use std::sync::Arc;
use systemprompt::identifiers::{EvalCampaignId, EvalExperimentId, ResourceRevisionId};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CampaignLaunchForm {
    template_id: EvalExperimentId,
    candidate_revision_id: ResourceRevisionId,
    operation_key: String,
}

pub(crate) async fn launch(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path(id): Path<EvalCampaignId>,
    headers: HeaderMap,
    Form(form): Form<CampaignLaunchForm>,
) -> AdminHtmlResult<Redirect> {
    require_admin(&user)?;
    crate::handlers::evaluation_experiments::require_write_origin(&headers)?;
    let diagnostic_key = format!(
        "dashboard-launch:{}",
        systemprompt::evaluation::experiments::content_digest(&form.operation_key)?
    );
    let result: AdminHtmlResult<Redirect> =
        async {
            let campaign = state.campaigns.get(&state.owner, &id).await?;
            let mut spec = state
                .experiments
                .get(&state.owner, &form.template_id)
                .await?
                .experiment
                .spec;
            if spec.variants.len() != 2 {
                return Err(AdminError::BadRequest(
                    "Choose a paired experiment template".to_owned(),
                )
                .into());
            }
            spec.variants[0].skill_bundle_digest = state
                .optimization
                .register_workspace(&state.owner, &campaign.policy.baseline_revision_id)
                .await?;
            spec.variants[1].skill_bundle_digest = state
                .optimization
                .register_workspace(&state.owner, &form.candidate_revision_id)
                .await?;
            spec.claim_independent_improvement = false;
            let mut cases = Vec::new();
            for case in &spec.cases {
                if let systemprompt::evaluation::experiments::resources::ResourceContent::Case(
                    content,
                ) = state.revisions.get(&state.owner, case).await?
                    && content.partition
                        == systemprompt::evaluation::experiments::resources::Partition::Development
                {
                    cases.push(case.clone());
                }
            }
            spec.cases = cases;
            let experiment = state
                .optimization
                .launch(
                    &state.owner,
                    &user.user_id,
                    &systemprompt::evaluation::repository::experiments::CampaignExperiment {
                        campaign_id: id,
                        idempotency_key: form.operation_key,
                        spec,
                    },
                )
                .await?;
            Ok(Redirect::to(&format!(
                "/admin/analysis/evaluations/{experiment}"
            )))
        }
        .await;
    if result.is_err() {
        record_setup_failure(&state, &user, &diagnostic_key).await?;
    }
    result
}

async fn record_setup_failure(
    state: &EvaluationState,
    user: &UserContext,
    diagnostic_key: &str,
) -> Result<(), AdminError> {
    state
        .campaigns
        .record_diagnostic(
            &state.owner,
            systemprompt::evaluation::campaigns::diagnostics::DiagnosticRecord {
                actor: &user.user_id,
                campaign: None,
                operation: diagnostic_key,
                stage: systemprompt::evaluation::campaigns::diagnostics::DiagnosticStage::Setup,
                code:
                    systemprompt::evaluation::campaigns::diagnostics::DiagnosticCode::InvalidInput,
            },
        )
        .await?;
    Ok(())
}
