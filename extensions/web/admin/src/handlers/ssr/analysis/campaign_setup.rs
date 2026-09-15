//! Campaign policy setup retains actionable failures before a campaign exists.
use super::require_admin;
use crate::error::{AdminError, AdminHtmlResult};
use crate::routes::evaluation_state::EvaluationState;
use crate::routes::managed_state::ManagedState;
use crate::types::UserContext;
use axum::Extension;
use axum::extract::Form;
use axum::http::HeaderMap;
use axum::response::Redirect;
use serde::Deserialize;
use std::sync::Arc;
use systemprompt::evaluation::campaigns::{CampaignPolicy, OptimizationObjective};
use systemprompt::identifiers::{ManagedResourceId, ResourceRevisionId};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CampaignForm {
    name: String,
    baseline: String,
    budget_dollars: String,
    objective: OptimizationObjective,
    minimum_quality_milli: u32,
    minimum_pairs: u32,
    maximum_iterations: u32,
    #[serde(default)]
    automatic: bool,
    operation_key: String,
}

pub(crate) async fn create(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Extension(managed): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
    Form(form): Form<CampaignForm>,
) -> AdminHtmlResult<Redirect> {
    require_admin(&user)?;
    crate::handlers::evaluation_experiments::require_write_origin(&headers)?;
    let diagnostic_key = format!(
        "dashboard-create:{}",
        systemprompt::evaluation::experiments::content_digest(&form.operation_key)?
    );
    let result: AdminHtmlResult<Redirect> = async {
        let (resource, baseline) = form
            .baseline
            .split_once('|')
            .ok_or_else(|| AdminError::BadRequest("Choose an imported baseline".to_owned()))?;
        let resource_id = ManagedResourceId::new(resource);
        let baseline_revision_id = ResourceRevisionId::new(baseline);
        if managed
            .repository
            .revision_resource(&managed.owner, &baseline_revision_id)
            .await?
            != resource_id
        {
            return Err(AdminError::BadRequest(
                "Baseline does not belong to this resource".to_owned(),
            )
            .into());
        }
        let cap = dollars_to_microdollars(&form.budget_dollars)?;
        let budget_id = state
            .budgets
            .create_shared(
                &state.owner,
                &format!("campaign:{}", form.operation_key),
                cap,
            )
            .await?;
        let policy = CampaignPolicy {
            name: form.name,
            resource_id,
            baseline_revision_id,
            budget_id,
            objective: form.objective,
            minimum_quality_milli: form.minimum_quality_milli,
            minimum_pairs: form.minimum_pairs,
            maximum_iterations: form.maximum_iterations,
            automatic: form.automatic,
        };
        state
            .campaigns
            .create(&state.owner, &user.user_id, &form.operation_key, &policy)
            .await?;
        Ok(Redirect::to("/admin/analysis/campaigns"))
    }
    .await;
    if result.is_err() {
        state
            .campaigns
            .record_diagnostic(
                &state.owner,
                systemprompt::evaluation::campaigns::diagnostics::DiagnosticRecord {
                    actor: &user.user_id,
                    campaign: None,
                    operation: &diagnostic_key,
                    stage: systemprompt::evaluation::campaigns::diagnostics::DiagnosticStage::Setup,
                    code: systemprompt::evaluation::campaigns::diagnostics::DiagnosticCode::InvalidInput,
                },
            )
            .await?;
    }
    result
}

fn dollars_to_microdollars(value: &str) -> Result<i64, AdminError> {
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    let invalid = || {
        AdminError::BadRequest(
            "Enter a positive USD budget with at most two decimal places".to_owned(),
        )
    };
    if whole.is_empty()
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || fraction.len() > 2
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(invalid());
    }
    let dollars = whole.parse::<i64>().map_err(|_error| invalid())?;
    let cents = format!("{fraction:0<2}")
        .parse::<i64>()
        .map_err(|_error| invalid())?;
    dollars
        .checked_mul(1_000_000)
        .and_then(|amount| amount.checked_add(cents * 10_000))
        .filter(|amount| *amount > 0)
        .ok_or_else(invalid)
}
