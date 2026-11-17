//! `PUT /runs/{id}`: the owner device upserts a run summary — its state,
//! reason and timestamps, never the conversation. Another device of the same
//! person answers `403 not_owner_device`.

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::error::RoutinesError;
use crate::ids::{RoutineId, RunId, RunState};
use crate::state::RoutinesState;
use crate::store::{self, PutRun, RunOutcome, RunRecord};

const MAX_REASON: usize = 500;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RunBody {
    routine_id: RoutineId,
    device: String,
    state: RunState,
    reason: Option<String>,
    started_at: Option<DateTime<Utc>>,
    ended_at: Option<DateTime<Utc>>,
}

pub async fn put(
    State(state): State<RoutinesState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<RunRecord>, RoutinesError> {
    let desktop = super::caller(&state, &headers)?;
    let id = RunId::parse(&id).map_err(|error| {
        tracing::debug!(%error, "run id in the path is malformed");
        RoutinesError::NotFound
    })?;
    let body: RunBody = serde_json::from_slice(&body).map_err(|error| {
        tracing::debug!(%error, "run body is not a run summary");
        RoutinesError::BadRequest(
            "the body is {routine_id, device, state, reason?, started_at?, ended_at?}",
        )
    })?;
    if body
        .reason
        .as_ref()
        .is_some_and(|reason| reason.chars().count() > MAX_REASON)
    {
        return Err(RoutinesError::BadRequest(
            "reason is at most 500 characters",
        ));
    }
    let outcome = store::put_run(
        &state.pool,
        &PutRun {
            user: &desktop.user,
            id: &id,
            routine_id: &body.routine_id,
            device: &body.device,
            state: body.state,
            reason: body.reason.as_deref(),
            started_at: body.started_at,
            ended_at: body.ended_at,
            now: Utc::now(),
        },
    )
    .await?;
    match outcome {
        RunOutcome::Written(record) => Ok(Json(record)),
        RunOutcome::RoutineNotFound => Err(RoutinesError::NotFound),
        RunOutcome::NotOwnerDevice => Err(RoutinesError::NotOwnerDevice),
        RunOutcome::RoutineMismatch => Err(RoutinesError::RunConflict),
    }
}
