//! Astound campaign presentation delegates policy and persistence to core.

use axum::Extension;
use axum::extract::{Form, Path, Query};
use axum::http::HeaderMap;
use axum::response::{Redirect, Response};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use systemprompt::evaluation::campaigns::repository::CampaignRecord;
use systemprompt::identifiers::{EvalCampaignId, EvalExperimentId, ResourceRevisionId};

use crate::error::{AdminError, AdminHtmlResult};
use crate::routes::evaluation_state::EvaluationState;
use crate::routes::managed_state::ManagedState;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

#[path = "campaign_figures.rs"]
mod figures;
use figures::{MeasuredUsage, ResourceOption, measured_by_resource, resource_options};

#[path = "campaign_setup.rs"]
mod setup;
pub(crate) use setup::create;

#[derive(Serialize)]
struct CampaignDashboard {
    page: &'static str,
    title: &'static str,
    campaigns: Vec<CampaignView>,
    diagnostics: Vec<systemprompt::evaluation::campaigns::diagnostics::CampaignDiagnostic>,
    capabilities: Vec<systemprompt::evaluation::repository::experiments::CampaignAvailability>,
    templates: Vec<systemprompt::evaluation::experiments::records::ExperimentRecord>,
    resources: Vec<ResourceOption>,
    preselected_baseline: String,
    operation_key: String,
}

#[derive(Serialize)]
struct CampaignView {
    #[serde(flatten)]
    campaign: CampaignRecord,
    experiments: Vec<EvalExperimentId>,
    candidates: Vec<systemprompt::marketplace::managed::RevisionSummary>,
    resource_key: String,
    measured: Option<MeasuredUsage>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CampaignPageQuery {
    baseline: Option<String>,
}

#[expect(
    clippy::too_many_arguments,
    reason = "axum extractor list; the router decides the arity, not this signature"
)]
pub(crate) async fn page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Extension(managed): Extension<Arc<ManagedState>>,
    Query(query): Query<CampaignPageQuery>,
) -> AdminHtmlResult<Response> {
    require_admin(&user)?;
    let all_resources = managed.repository.list_resources(&managed.owner, 0).await?;
    let measured = measured_by_resource(&managed, &all_resources).await?;
    let preselected_baseline = query.baseline.unwrap_or_default();
    let resources = resource_options(&all_resources, &measured, &preselected_baseline);
    let mut campaigns = Vec::new();
    for campaign in state.campaigns.list(&state.owner, None).await? {
        let resource_key = all_resources
            .iter()
            .find(|r| r.id == campaign.policy.resource_id)
            .map(|r| r.resource_key.clone())
            .unwrap_or_default();
        campaigns.push(CampaignView {
            experiments: state
                .campaigns
                .list_experiments(&state.owner, &campaign.id)
                .await?,
            candidates: managed
                .repository
                .list_revisions(&managed.owner, &campaign.policy.resource_id, 0)
                .await?,
            measured: measured.get(campaign.policy.resource_id.as_str()).cloned(),
            resource_key,
            campaign,
        });
    }
    let templates = state.experiments.list(&state.owner).await?;
    let capabilities = templates
        .iter()
        .map(|item| state.experiments.execution_availability(&item.spec))
        .collect();
    let data = CampaignDashboard {
        diagnostics: state
            .campaigns
            .diagnostics(&state.owner, None, None, 100)
            .await?,
        capabilities,
        page: "analysis-campaigns",
        title: "Optimization campaigns",
        campaigns,
        templates,
        resources,
        preselected_baseline,
        operation_key: uuid::Uuid::new_v4().to_string(),
    };
    Ok(super::super::render_typed_page(
        &engine,
        "analysis-campaigns",
        &data,
        &user,
        &marketplace,
    ))
}

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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AttachForm {
    experiment_id: EvalExperimentId,
}

pub(crate) async fn attach(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path(id): Path<EvalCampaignId>,
    headers: HeaderMap,
    Form(form): Form<AttachForm>,
) -> AdminHtmlResult<Redirect> {
    require_admin(&user)?;
    crate::handlers::evaluation_experiments::require_write_origin(&headers)?;
    state
        .optimization
        .attach(&state.owner, &user.user_id, &id, &form.experiment_id)
        .await?;
    Ok(Redirect::to("/admin/analysis/campaigns"))
}

pub(crate) async fn report(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path((id, experiment)): Path<(EvalCampaignId, EvalExperimentId)>,
) -> Result<impl axum::response::IntoResponse, AdminError> {
    require_admin(&user)?;
    let report = state
        .optimization
        .report(&state.owner, &id, &experiment)
        .await?;
    Ok((
        [
            ("cache-control", "no-store"),
            (
                "content-disposition",
                "attachment; filename=campaign-report.json",
            ),
        ],
        axum::Json(report),
    ))
}

fn require_admin(user: &UserContext) -> Result<(), AdminError> {
    if !user.is_admin {
        return Err(AdminError::Forbidden(
            "Administrator access required".to_owned(),
        ));
    }
    Ok(())
}
