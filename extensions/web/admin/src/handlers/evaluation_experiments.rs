//! Authenticated experiment authoring and owner-scoped evidence of queued work.

use crate::error::{AdminError, AdminResult};
use crate::types::UserContext;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::evaluation::experiments::records::{ExperimentDetail, ExperimentRecord};
use systemprompt::evaluation::experiments::resources::ResourceContent;
use systemprompt::evaluation::experiments::{
    ExperimentRepository, ExperimentSpec, RevisionRepository,
};
use systemprompt::identifiers::{EvalExperimentId, EvalRevisionId};

#[derive(Debug)]
pub(crate) struct EvaluationState {
    experiments: ExperimentRepository,
    revisions: RevisionRepository,
}

impl EvaluationState {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self {
            experiments: ExperimentRepository::new(pool.clone()),
            revisions: RevisionRepository::new(pool),
        }
    }
}

fn require_write_origin(headers: &HeaderMap) -> AdminResult<()> {
    let profile = systemprompt::config::ProfileBootstrap::get().map_err(AdminError::internal)?;
    let expected = url::Url::parse(&profile.server.api_external_url)
        .map_err(AdminError::internal)?
        .origin()
        .ascii_serialization();
    if headers.get("origin").and_then(|v| v.to_str().ok()) != Some(expected.as_str()) {
        return Err(AdminError::Forbidden(
            "Same-origin browser request required".to_owned(),
        ));
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreateExperiment {
    idempotency_key: String,
    spec: ExperimentSpec,
}

#[derive(Debug, Serialize)]
pub(crate) struct ExperimentCreated {
    id: EvalExperimentId,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreateRevision {
    key: String,
    content: ResourceContent,
}

#[derive(Debug, Serialize)]
pub(crate) struct RevisionCreated {
    id: EvalRevisionId,
}

pub(crate) async fn list(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
) -> AdminResult<Json<Vec<ExperimentRecord>>> {
    Ok(Json(state.experiments.list(&user.user_id).await?))
}

pub(crate) async fn show(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path(id): Path<String>,
) -> AdminResult<Json<ExperimentDetail>> {
    Ok(Json(
        state
            .experiments
            .get(&user.user_id, &EvalExperimentId::new(id))
            .await?,
    ))
}

pub(crate) async fn create(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    headers: HeaderMap,
    Json(input): Json<CreateExperiment>,
) -> AdminResult<(StatusCode, Json<ExperimentCreated>)> {
    require_write_origin(&headers)?;
    let id = state
        .experiments
        .create(&user.user_id, &input.idempotency_key, &input.spec)
        .await?;
    Ok((StatusCode::ACCEPTED, Json(ExperimentCreated { id })))
}

pub(crate) async fn cancel(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> AdminResult<StatusCode> {
    require_write_origin(&headers)?;
    state
        .experiments
        .cancel(&user.user_id, &EvalExperimentId::new(id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn create_revision(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    headers: HeaderMap,
    Json(input): Json<CreateRevision>,
) -> AdminResult<(StatusCode, Json<RevisionCreated>)> {
    require_write_origin(&headers)?;
    let id = state
        .revisions
        .create(&user.user_id, &input.key, &input.content)
        .await?;
    Ok((StatusCode::CREATED, Json(RevisionCreated { id })))
}

pub(crate) async fn get_revision(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path(id): Path<String>,
) -> AdminResult<Json<ResourceContent>> {
    Ok(Json(
        state
            .revisions
            .get(&user.user_id, &EvalRevisionId::new(id))
            .await?,
    ))
}
