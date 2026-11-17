//! `PUT /threads/{id}` and `DELETE /threads/{id}`.
//!
//! A `PUT` without `If-Match` creates the thread at revision 1; an id that
//! already exists answers `409` with the caller's row, or `id_taken` when it
//! is another user's (whose row is never shown). With `If-Match: <revision>`
//! it renames, archives, unarchives or changes the model. A `DELETE` needs
//! `If-Match` (`428` without it) and answers the tombstone. A stale revision,
//! or a write to a tombstone, answers `409` with the current row; a thread
//! that is not the caller's is `404`, never `403`.

use axum::body::Bytes;
use axum::extract::rejection::{BytesRejection, PathRejection};
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::Response;
use dynwork_relay::auth::AuthError;

use crate::auth::Caller;
use crate::error::ChatsError;
use crate::http::params::{ThreadInput, if_match};
use crate::http::response;
use crate::ids::ThreadId;
use crate::state::ChatsState;
use crate::store::{CreateOutcome, NewThread, ThreadEdit, WriteOutcome};

pub async fn put(
    caller: Caller,
    State(state): State<ChatsState>,
    path: Result<Path<String>, PathRejection>,
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> Result<Response, ChatsError> {
    let Path(id) = path?;
    let id: ThreadId = id.parse()?;
    let revision = if_match(&headers)?;
    let input = ThreadInput::parse(&body?)?;
    match revision {
        None => create(&state, &caller, id, input).await,
        Some(expected_revision) => {
            let edit = ThreadEdit {
                expected_revision,
                patch: input.into_patch(),
            };
            let outcome = state
                .store
                .update_thread(&caller.user, &id, &edit, state.clock.now())
                .await?;
            written(&state, &caller, &id, outcome).await
        },
    }
}

pub async fn delete(
    caller: Caller,
    State(state): State<ChatsState>,
    path: Result<Path<String>, PathRejection>,
    headers: HeaderMap,
) -> Result<Response, ChatsError> {
    let Path(id) = path?;
    let id: ThreadId = id.parse()?;
    let expected = if_match(&headers)?.ok_or(ChatsError::PreconditionRequired)?;
    let outcome = state
        .store
        .delete_thread(&caller.user, &id, expected, state.clock.now())
        .await?;
    written(&state, &caller, &id, outcome).await
}

async fn create(
    state: &ChatsState,
    caller: &Caller,
    id: ThreadId,
    input: ThreadInput,
) -> Result<Response, ChatsError> {
    if input.archived == Some(true) {
        return Err(ChatsError::BadRequest("a new thread cannot be archived"));
    }
    let patch = input.into_patch();
    let thread = NewThread {
        id,
        title: patch.title.unwrap_or_default(),
        model: patch.model,
    };
    match state
        .store
        .create_thread(&caller.user, &thread, state.clock.now())
        .await?
    {
        CreateOutcome::Created(record) => Ok(response::thread(&record)),
        CreateOutcome::Exists(record) => Err(ChatsError::Conflict(Box::new(record))),
        CreateOutcome::IdTaken => Err(ChatsError::IdTaken),
        CreateOutcome::UnknownUser => Err(AuthError::UnknownUser.into()),
    }
}

async fn written(
    state: &ChatsState,
    caller: &Caller,
    id: &ThreadId,
    outcome: WriteOutcome,
) -> Result<Response, ChatsError> {
    match outcome {
        WriteOutcome::Done(record) => Ok(response::thread(&record)),
        WriteOutcome::Conflict(record) => Err(ChatsError::Conflict(Box::new(record))),
        WriteOutcome::NotFound => match state.store.get_thread(&caller.user, id).await? {
            Some(tombstone) if tombstone.deleted_at.is_some() => {
                Err(ChatsError::Conflict(Box::new(tombstone)))
            },
            _ => Err(ChatsError::NotFound),
        },
    }
}
