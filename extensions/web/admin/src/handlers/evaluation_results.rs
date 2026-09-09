//! Owner-scoped execution manifests and verified artifact downloads.

use crate::error::AdminResult;
use crate::routes::evaluation_state::EvaluationState;
use crate::types::UserContext;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use std::sync::Arc;
use systemprompt::identifiers::EvalExecutionId;

pub(crate) async fn evidence(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path(id): Path<EvalExecutionId>,
) -> AdminResult<impl IntoResponse> {
    let evidence = state.evidence.get(&user.user_id, &id).await?;
    Ok(([("cache-control", "no-store")], Json(evidence)))
}

pub(crate) async fn artifacts(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path(id): Path<EvalExecutionId>,
) -> AdminResult<impl IntoResponse> {
    let artifacts = state.evidence.get_artifacts(&user.user_id, &id).await?;
    Ok(([("cache-control", "no-store")], Json(artifacts)))
}
