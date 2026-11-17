//! `PUT /routines/{id}` and `DELETE /routines/{id}`.
//!
//! A `PUT` without `If-Match` creates the routine; with it, it replaces the
//! definition of that revision. A `DELETE` needs `If-Match` and leaves a
//! tombstone. Both answer `200` with the routine at its new revision (and an
//! `ETag`), or `409 revision_conflict` with the current row.
//!
//! A definition with a `template` links the routine to that template's
//! revision. A new link needs a published template the caller sees and no
//! other live routine of it (`409 already_activated` with that routine); every
//! linked write keeps the fields the named revision locks (`409
//! template_locked` with the current row). A routine linked to a published
//! template that is mandatory for the caller is not deleted (`409
//! template_mandatory`); pausing it is an ordinary write.

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, header};
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use serde::Deserialize;
// JSON: protocol boundary — the definition is DynWork's, stored as it sent it.
use serde_json::Value;
use systemprompt::identifiers::UserId;

use crate::error::RoutinesError;
use crate::ids::{RoutineId, TemplateId};
use crate::state::RoutinesState;
use crate::store::{self, DeleteRoutine, PutLink, PutRoutine, TemplateState, WriteOutcome};
use crate::template::{self, Link};
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
    let link = template::link(&body.definition)?;
    let link = match &link {
        Some(link) => Some(check_link(&state, &desktop.user, link, &body.definition).await?),
        None => None,
    };
    let outcome = store::put_routine(
        &state.pool,
        &PutRoutine {
            user: &desktop.user,
            id: &id,
            expected,
            owner_device: body.owner_device.as_deref(),
            owner_name: body.owner_name.as_deref(),
            definition: &body.definition,
            link,
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
    let mandatory = mandatory_for(&state, &desktop.user).await?;
    let outcome = store::delete_routine(
        &state.pool,
        &DeleteRoutine {
            user: &desktop.user,
            id: &id,
            expected: Some(expected),
            mandatory: &mandatory,
            now: Utc::now(),
        },
    )
    .await?;
    let written = written(outcome)?;
    super::hint(&state.pool, &desktop.user, peer).await;
    Ok(answer(written))
}

// JSON: protocol boundary — only the definition's locked paths are read.
// Why: the revision is read here, outside the write's transaction, because a
// published revision never changes; whether the link is new is only known
// inside it, so whether the template is open to the caller travels along.
async fn check_link<'a>(
    state: &RoutinesState,
    user: &UserId,
    link: &'a Link,
    definition: &Value,
) -> Result<PutLink<'a>, RoutinesError> {
    let revision = store::find_template_revision(&state.pool, &link.id, link.revision)
        .await?
        .ok_or(RoutinesError::BadRequest(
            "the template has no such revision",
        ))?;
    let open = match store::find_template(&state.pool, &link.id).await? {
        Some(found) if found.state == TemplateState::Published => {
            let allowed = super::templates::visible_plugins(state, user, &found.plugins).await?;
            found.plugins.iter().any(|plugin| allowed.contains(plugin))
        },
        _ => false,
    };
    Ok(PutLink {
        template: &link.id,
        revision: link.revision,
        keeps_locks: template::keeps_locks(&revision.body, definition),
        open,
    })
}

// Why: a routine is not deleted while its template is published, mandatory
// and seen by its person, so only those templates' access is resolved.
async fn mandatory_for(
    state: &RoutinesState,
    user: &UserId,
) -> Result<Vec<TemplateId>, RoutinesError> {
    let templates: Vec<_> = store::list_templates(&state.pool)
        .await?
        .into_iter()
        .filter(|template| template.state == TemplateState::Published && template.mandatory)
        .collect();
    if templates.is_empty() {
        return Ok(Vec::new());
    }
    let anchors: Vec<String> = templates
        .iter()
        .flat_map(|template| template.plugins.iter().cloned())
        .collect();
    let allowed = super::templates::visible_plugins(state, user, &anchors).await?;
    Ok(templates
        .into_iter()
        .filter(|template| {
            template
                .plugins
                .iter()
                .any(|plugin| allowed.contains(plugin))
        })
        .map(|template| template.id)
        .collect())
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
        WriteOutcome::AlreadyActivated(routine) => {
            Err(RoutinesError::AlreadyActivated(Box::new(routine.into())))
        },
        WriteOutcome::TemplateLocked(current) => Err(RoutinesError::TemplateLocked(
            current.map(|current| Box::new(current.into())),
        )),
        WriteOutcome::TemplateUnavailable => Err(RoutinesError::BadRequest(
            "the template is not open to a new activation",
        )),
        WriteOutcome::Mandatory => Err(RoutinesError::TemplateMandatory),
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
