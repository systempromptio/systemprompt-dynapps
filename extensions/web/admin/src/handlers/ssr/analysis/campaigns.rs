//! Astound campaign presentation delegates policy and persistence to core.

use axum::Extension;
use axum::extract::{Form, Path, Query, State};
use axum::http::HeaderMap;
use axum::response::{Redirect, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::evaluation::campaigns::repository::CampaignRecord;
use systemprompt::identifiers::{EvalCampaignId, EvalExperimentId};

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

#[path = "campaign_launch.rs"]
mod launching;
pub(crate) use launching::launch;

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
    preselected_marketplace: Option<String>,
    form_open: bool,
    operation_key: String,
    kpis: CampaignKpis,
}

#[derive(Serialize)]
struct CampaignKpis {
    total: usize,
    active: usize,
    stale: usize,
    resources: usize,
}

#[derive(Serialize)]
struct CampaignView {
    #[serde(flatten)]
    campaign: CampaignRecord,
    experiments: Vec<EvalExperimentId>,
    candidates: Vec<systemprompt::marketplace::managed::RevisionSummary>,
    resource_key: String,
    measured: Option<MeasuredUsage>,
    composed_hash_short: Option<String>,
    served_generation: Option<i64>,
    stale: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CampaignPageQuery {
    baseline: Option<String>,
    resource: Option<String>,
    marketplace: Option<String>,
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
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<CampaignPageQuery>,
) -> AdminHtmlResult<Response> {
    require_admin(&user)?;
    let all_resources = managed
        .repository
        .list_resources(&managed.owner, 0)
        .await?
        .items;
    let measured = measured_by_resource(&managed, &all_resources).await?;
    let preselected_baseline = query
        .baseline
        .clone()
        .or_else(|| baseline_of(&all_resources, query.resource.as_deref()))
        .unwrap_or_default();
    let resources = resource_options(&all_resources, &measured, &preselected_baseline);
    let preselected_baseline_present = !preselected_baseline.is_empty();
    let served = crate::repositories::analysis::publication_heads::list_publication_heads(
        &pool,
        &managed.owner,
    )
    .await?;
    let campaigns = campaign_views(&state, &managed, &all_resources, &measured, &served).await?;
    let templates = state.experiments.list(&state.owner).await?;
    let capabilities = templates
        .iter()
        .map(|item| state.experiments.execution_availability(&item.spec))
        .collect();
    let kpis = CampaignKpis {
        total: campaigns.len(),
        active: campaigns
            .iter()
            .filter(|view| {
                view.campaign.status == systemprompt::evaluation::models::CampaignStatus::Active
            })
            .count(),
        stale: campaigns.iter().filter(|view| view.stale).count(),
        resources: resources.len(),
    };
    let data = CampaignDashboard {
        diagnostics: state
            .campaigns
            .diagnostics(&state.owner, None, None, 100)
            .await?,
        capabilities,
        page: "analysis-campaigns",
        title: "Campaigns",
        campaigns,
        templates,
        resources,
        preselected_baseline,
        form_open: preselected_baseline_present,
        preselected_marketplace: query.marketplace,
        operation_key: uuid::Uuid::new_v4().to_string(),
        kpis,
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

async fn campaign_views(
    state: &EvaluationState,
    managed: &ManagedState,
    all_resources: &[systemprompt::marketplace::managed::ResourceSummary],
    measured: &std::collections::HashMap<String, MeasuredUsage>,
    served: &[crate::repositories::analysis::publication_heads::PublicationHead],
) -> Result<Vec<CampaignView>, AdminError> {
    let mut campaigns = Vec::new();
    for campaign in state.campaigns.list(&state.owner, None).await? {
        let resource_key = all_resources
            .iter()
            .find(|r| r.id == campaign.policy.resource_id)
            .map(|r| r.resource_key.clone())
            .unwrap_or_default();
        let served_generation = served
            .iter()
            .find(|head| head.resource_id == campaign.policy.resource_id)
            .map(|head| head.generation);
        campaigns.push(CampaignView {
            composed_hash_short: campaign
                .composed_hash
                .as_deref()
                .map(|hash| hash.chars().take(12).collect()),
            // Why: a campaign measures the generation it was created against, so
            // once the served generation moves past it the comparison stays valid
            // evidence about a baseline that users have stopped receiving.
            stale: campaign
                .publication_generation
                .is_some_and(|pinned| served_generation.is_some_and(|now| now > pinned)),
            served_generation,
            experiments: state
                .campaigns
                .list_experiments(&state.owner, &campaign.id)
                .await?,
            candidates: managed
                .repository
                .list_revisions(&managed.owner, &campaign.policy.resource_id, 0)
                .await?
                .items,
            measured: measured.get(campaign.policy.resource_id.as_str()).cloned(),
            resource_key,
            campaign,
        });
    }
    Ok(campaigns)
}

// Why: the Skills page links here with the resource it was looking at, but the
// form's option value is `resource|revision`; the revision is the resource's
// latest, which only this side knows.
fn baseline_of(
    resources: &[systemprompt::marketplace::managed::ResourceSummary],
    resource: Option<&str>,
) -> Option<String> {
    let wanted = resource?;
    resources
        .iter()
        .find(|item| item.id.as_str() == wanted)
        .and_then(|item| {
            item.latest_revision
                .as_ref()
                .map(|revision| format!("{}|{revision}", item.id))
        })
}

fn require_admin(user: &UserContext) -> Result<(), AdminError> {
    if !user.is_admin {
        return Err(AdminError::Forbidden(
            "Administrator access required".to_owned(),
        ));
    }
    Ok(())
}
