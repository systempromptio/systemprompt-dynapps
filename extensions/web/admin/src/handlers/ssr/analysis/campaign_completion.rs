//! Guided independent confirmation and on-demand inventory sync.
use crate::error::{AdminError, AdminHtmlResult};
use crate::routes::evaluation_state::EvaluationState;
use crate::routes::managed_state::ManagedState;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::Extension;
use axum::extract::{Form, Path};
use axum::http::HeaderMap;
use axum::response::{Redirect, Response};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use systemprompt::identifiers::{
    EvalCampaignId, EvalExperimentId, EvalHoldoutProposalId, EvalRevisionId,
};
use systemprompt::system::optimization::holdout::{
    ConfirmHoldout, HoldoutConfirmationTarget, HoldoutReview, PrepareHoldout,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HoldoutForm {
    development_experiment_id: EvalExperimentId,
    holdout_dataset_id: EvalRevisionId,
    idempotency_key: String,
}
#[derive(Serialize)]
struct ReviewPage {
    page: &'static str,
    title: &'static str,
    review: HoldoutReview,
}
#[expect(
    clippy::too_many_arguments,
    reason = "axum extractor list; the router decides the arity, not this signature"
)]
pub(crate) async fn prepare(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path(campaign): Path<EvalCampaignId>,
    headers: HeaderMap,
    Form(form): Form<HoldoutForm>,
) -> AdminHtmlResult<Response> {
    require_write(&user, &headers)?;
    let review = state
        .optimization
        .prepare_holdout(
            &state.owner,
            &user.user_id,
            &campaign,
            &PrepareHoldout {
                development_experiment_id: form.development_experiment_id,
                holdout_dataset_id: form.holdout_dataset_id,
                idempotency_key: form.idempotency_key,
            },
        )
        .await?;
    Ok(super::super::render_typed_page(
        &engine,
        "analysis-holdout",
        &ReviewPage {
            page: "analysis-campaigns",
            title: "Review independent holdout",
            review,
        },
        &user,
        &marketplace,
    ))
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConfirmationForm {
    spec_digest: String,
    #[serde(default)]
    confirm_independent_holdout: bool,
}
pub(crate) async fn confirm(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path((campaign, proposal)): Path<(EvalCampaignId, EvalHoldoutProposalId)>,
    headers: HeaderMap,
    Form(form): Form<ConfirmationForm>,
) -> AdminHtmlResult<Redirect> {
    require_write(&user, &headers)?;
    state
        .optimization
        .confirm_holdout(
            &state.owner,
            &HoldoutConfirmationTarget {
                actor: &user.user_id,
                campaign: &campaign,
                id: &proposal,
            },
            &ConfirmHoldout {
                spec_digest: form.spec_digest,
                confirm_independent_holdout: form.confirm_independent_holdout,
            },
        )
        .await?;
    Ok(Redirect::to("/admin/analysis/campaigns"))
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SyncForm {
    #[serde(default)]
    back: Option<String>,
}
// Why: the form says where it came from; anything but an analysis page is
// an open redirect and lands on the portfolio instead.
fn validated_back(back: Option<&str>) -> &'static str {
    match back {
        Some("/admin/analysis/campaigns") => "/admin/analysis/campaigns",
        _ => "/admin/analysis/resources",
    }
}
// Why: the same pass the scheduler runs each minute, on demand — refresh the
// inventory, then publish the latest configured revision of every skill.
pub(crate) async fn sync_inventory(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
    Form(form): Form<SyncForm>,
) -> AdminHtmlResult<Redirect> {
    require_write(&user, &headers)?;
    let root = crate::handlers::shared::get_services_path()?;
    let services = systemprompt::loader::ConfigLoader::load().map_err(AdminError::internal)?;
    let service =
        systemprompt::marketplace::inventory::InventoryService::new(state.repository.clone());
    service.refresh(&state.owner, &root, &services).await?;
    service
        .publish_latest(
            &systemprompt::marketplace::inventory::BaselineScope {
                owner: &state.owner,
                actor: &user.user_id,
                root: &root,
                services: &services,
            },
            &mut systemprompt::marketplace::inventory::PublishGuard::default(),
        )
        .await?;
    Ok(Redirect::to(validated_back(form.back.as_deref())))
}
fn require_write(user: &UserContext, headers: &HeaderMap) -> Result<(), AdminError> {
    if !user.is_admin {
        return Err(AdminError::Forbidden(
            "Administrator access required".to_owned(),
        ));
    }
    crate::handlers::evaluation_experiments::require_write_origin(headers)
}
