//! `PUT /routines/{id}` and `DELETE /routines/{id}`.
//!
//! A `PUT` without `If-Match` creates the routine; with it, it replaces the
//! definition of that revision. A `DELETE` needs `If-Match` and leaves a
//! tombstone. Both answer `200` with the routine at its new revision (and an
//! `ETag`), or `409 revision_conflict` with the current row.

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, header};
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use serde::Deserialize;
// JSON: protocol boundary — the definition is DynWork's, stored as it sent it.
use serde_json::Value;

use crate::error::RoutinesError;
use crate::ids::RoutineId;
use crate::state::RoutinesState;
use crate::store::{self, PutRoutine, WriteOutcome};
use crate::view::RoutineView;

const MAX_DEFINITION: usize = 64 * 1024;
const MAX_TEXT: usize = 128;

// JSON: protocol boundary — the definition is DynWork's, stored as it sent it.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PutBody {
    owner_device: Option<String>,
    owner_name: Option<String>,
    definition: Value,
}

pub async fn put(
    State(state): State<RoutinesState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, RoutinesError> {
    let desktop = super::caller(&state, &headers)?;
    let id = routine_id(&id)?;
    let expected = super::if_match(&headers)?;
    let peer = super::caller_peer(&headers)?;
    let body: PutBody = serde_json::from_slice(&body).map_err(|error| {
        tracing::debug!(%error, "routine body is not a routine");
        RoutinesError::BadRequest("the body is {owner_device?, owner_name?, definition}")
    })?;
    check_body(&body)?;
    let outcome = store::put_routine(
        &state.pool,
        &PutRoutine {
            user: &desktop.user,
            id: &id,
            expected,
            owner_device: body.owner_device.as_deref(),
            owner_name: body.owner_name.as_deref(),
            definition: &body.definition,
            now: Utc::now(),
        },
    )
    .await?;
    let written = written(outcome)?;
    super::hint(&state.pool, &desktop.user, peer).await;
    Ok(answer(written))
}

pub async fn delete(
    State(state): State<RoutinesState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, RoutinesError> {
    let desktop = super::caller(&state, &headers)?;
    let id = routine_id(&id)?;
    let expected = super::if_match(&headers)?
        .ok_or(RoutinesError::BadRequest("If-Match is required to delete"))?;
    let peer = super::caller_peer(&headers)?;
    let outcome =
        store::delete_routine(&state.pool, &desktop.user, &id, Some(expected), Utc::now()).await?;
    let written = written(outcome)?;
    super::hint(&state.pool, &desktop.user, peer).await;
    Ok(answer(written))
}

fn routine_id(raw: &str) -> Result<RoutineId, RoutinesError> {
    RoutineId::parse(raw).map_err(|error| {
        tracing::debug!(%error, "routine id in the path is malformed");
        RoutinesError::NotFound
    })
}

fn written(outcome: WriteOutcome) -> Result<RoutineView, RoutinesError> {
    match outcome {
        WriteOutcome::Written(record) => Ok(record.into()),
        WriteOutcome::Conflict(current) => Err(RoutinesError::Conflict(Box::new(current.into()))),
        WriteOutcome::NotFound => Err(RoutinesError::NotFound),
        WriteOutcome::OwnerRequired => Err(RoutinesError::BadRequest(
            "owner_device is required to create a routine",
        )),
        WriteOutcome::UnknownUser => Err(dynwork_relay::auth::AuthError::UnknownUser.into()),
    }
}

fn answer(view: RoutineView) -> Response {
    let etag = HeaderValue::from(view.revision);
    let mut response = Json(view).into_response();
    response.headers_mut().insert(header::ETAG, etag);
    response
}

// Why: the page reads `name` and `enabled` from the definition; the rest is
// DynWork's.
fn check_body(body: &PutBody) -> Result<(), RoutinesError> {
    let too_long = |text: &Option<String>| {
        text.as_ref()
            .is_some_and(|text| text.chars().count() > MAX_TEXT)
    };
    if body.owner_device.as_ref().is_some_and(String::is_empty)
        || too_long(&body.owner_device)
        || too_long(&body.owner_name)
    {
        return Err(RoutinesError::BadRequest(
            "owner_device and owner_name are 1 to 128 characters",
        ));
    }
    let definition = &body.definition;
    let shaped = definition.is_object()
        && definition
            .get("name")
            .and_then(Value::as_str)
            .is_some_and(|name| !name.is_empty())
        && definition.get("enabled").is_some_and(Value::is_boolean);
    if !shaped {
        return Err(RoutinesError::BadRequest(
            "the definition is an object with a name and an enabled flag",
        ));
    }
    if definition.to_string().len() > MAX_DEFINITION {
        return Err(RoutinesError::BadRequest("the definition is over 64 KiB"));
    }
    Ok(())
}
