//! Guided independent confirmation and inventory-wide baseline preparation.
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
    EvalCampaignId, EvalExperimentId, EvalRevisionId, InventoryEntryId, TaskId,
};
use systemprompt::system::optimization::holdout::{ConfirmHoldout, HoldoutReview, PrepareHoldout};

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
    Path((campaign, proposal)): Path<(EvalCampaignId, String)>,
    headers: HeaderMap,
    Form(form): Form<ConfirmationForm>,
) -> AdminHtmlResult<Redirect> {
    require_write(&user, &headers)?;
    state
        .optimization
        .confirm_holdout(
            &state.owner,
            &user.user_id,
            &campaign,
            &proposal,
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
pub(crate) struct BaselineForm {
    operation_id: TaskId,
    after: Option<InventoryEntryId>,
}
#[derive(Serialize)]
struct BaselinePage {
    page: &'static str,
    title: &'static str,
    operation_id: TaskId,
    next_cursor: Option<InventoryEntryId>,
    items: Vec<systemprompt::marketplace::inventory::BaselineCapture>,
}
pub(crate) async fn baselines(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(state): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
    Form(form): Form<BaselineForm>,
) -> AdminHtmlResult<Response> {
    require_write(&user, &headers)?;
    let root = crate::handlers::shared::get_services_path()?;
    let services = systemprompt::loader::ConfigLoader::load().map_err(AdminError::internal)?;
    let service =
        systemprompt::marketplace::inventory::InventoryService::new(state.repository.clone());
    service.refresh(&state.owner, &root, &services).await?;
    let items = service
        .prepare_baselines(
            &state.owner,
            &user.user_id,
            &systemprompt::marketplace::inventory::BaselinePreparation {
                operation_id: form.operation_id.clone(),
                after: form.after,
                limit: 100,
            },
            &root,
            &services,
        )
        .await?;
    let next_cursor = if items.len() == 100 {
        items.last().map(|item| item.entry_id.clone())
    } else {
        None
    };
    Ok(super::super::render_typed_page(
        &engine,
        "analysis-baselines",
        &BaselinePage {
            page: "analysis-campaigns",
            title: "Inventory baseline preparation",
            operation_id: form.operation_id,
            next_cursor,
            items,
        },
        &user,
        &marketplace,
    ))
}
fn require_write(user: &UserContext, headers: &HeaderMap) -> Result<(), AdminError> {
    if !user.is_admin {
        return Err(AdminError::Forbidden(
            "Administrator access required".to_owned(),
        ));
    }
    crate::handlers::evaluation_experiments::require_write_origin(headers)
}
