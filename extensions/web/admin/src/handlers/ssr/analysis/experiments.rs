//! Owner-scoped experiment records expose progress without inferring quality.

use crate::error::{AdminError, AdminHtmlResult};
use crate::routes::evaluation_state::EvaluationState;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::Extension;
use axum::extract::Path;
use axum::response::Response;
use serde::Serialize;
use std::sync::Arc;
use systemprompt::evaluation::experiments::records::{ExperimentRecord, ExperimentStatus};
use systemprompt::identifiers::{EvalExecutionId, EvalExperimentId, EvalRevisionId};

#[derive(Serialize)]
struct ExperimentView {
    id: EvalExperimentId,
    name: String,
    status: ExperimentStatus,
    cases: usize,
    variants: usize,
    repetitions: u32,
    cap: String,
    settled: String,
    reserved: String,
    frozen: bool,
}

impl From<ExperimentRecord> for ExperimentView {
    fn from(row: ExperimentRecord) -> Self {
        Self {
            id: row.id,
            name: row.spec.name,
            status: row.status,
            cases: row.spec.cases.len(),
            variants: row.spec.variants.len(),
            repetitions: row.spec.repetitions,
            cap: super::dollars(row.accounting.cap),
            settled: super::dollars(row.accounting.settled),
            reserved: super::dollars(row.accounting.reserved),
            frozen: row.accounting.frozen,
        }
    }
}

pub(crate) async fn list_page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(state): Extension<Arc<EvaluationState>>,
) -> AdminHtmlResult<Response> {
    require_console(&user)?;
    let experiments = state
        .experiments
        .list(&user.user_id)
        .await
        .map_err(AdminError::from)?;
    let rendered = ExperimentIndexContext {
        page: "analysis-evaluations",
        title: "Evaluations",
        experiments: experiments.into_iter().map(ExperimentView::from).collect(),
    };
    Ok(super::super::render_typed_page(
        &engine,
        "analysis-evaluations",
        &rendered,
        &user,
        &marketplace,
    ))
}

#[derive(Serialize)]
struct ExecutionView {
    id: EvalExecutionId,
    case_revision: EvalRevisionId,
    variant: i32,
    repetition: i32,
    status: systemprompt::evaluation::experiments::records::ExecutionStatus,
    summary: Option<String>,
}

pub(crate) async fn detail_page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path(id): Path<EvalExperimentId>,
) -> AdminHtmlResult<Response> {
    require_console(&user)?;
    let detail = state
        .experiments
        .get(&user.user_id, &id)
        .await
        .map_err(AdminError::from)?;
    let specification =
        serde_json::to_string_pretty(&detail.experiment.spec).map_err(AdminError::internal)?;
    let rendered = ExperimentEvidenceContext {
        page: "analysis-evaluations",
        title: "Experiment evidence",
        experiment: detail.experiment.into(),
        specification,
        executions: detail
            .executions
            .into_iter()
            .map(|row| ExecutionView {
                id: row.id,
                case_revision: row.case_revision_id,
                variant: row.variant_index + 1,
                repetition: row.repetition + 1,
                status: row.status,
                summary: row.result.map(|result| result.summary),
            })
            .collect(),
    };
    Ok(super::super::render_typed_page(
        &engine,
        "analysis-experiment",
        &rendered,
        &user,
        &marketplace,
    ))
}

fn require_console(user: &UserContext) -> Result<(), AdminError> {
    if !user.is_console {
        return Err(AdminError::Forbidden("Console access required".to_owned()));
    }
    Ok(())
}

#[derive(Serialize)]
struct ExperimentIndexContext {
    page: &'static str,
    title: &'static str,
    experiments: Vec<ExperimentView>,
}

#[derive(Serialize)]
struct ExperimentEvidenceContext {
    page: &'static str,
    title: &'static str,
    experiment: ExperimentView,
    specification: String,
    executions: Vec<ExecutionView>,
}
