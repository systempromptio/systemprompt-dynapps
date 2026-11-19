//! `GET /threads/{id}/messages?before=<message id>&limit=<n>`.
//!
//! The thread's history, newest first. `more` says older messages remain and
//! `before` is the id to ask with for them. `limit` defaults to 50, at most
//! 200.

use axum::Json;
use axum::extract::rejection::{PathRejection, QueryRejection};
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::auth::Caller;
use crate::error::ChatsError;
use crate::http::params::page_size;
use crate::http::response::no_store;
use crate::http::wire::{WireMessages, message};
use crate::ids::{MessageId, ThreadId};
use crate::state::ChatsState;

pub const DEFAULT_PAGE: u32 = 50;
pub const MAX_MESSAGES: u32 = 200;

/// The query of a history page.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MessagesQuery {
    pub before: Option<String>,
    pub limit: Option<u32>,
}

pub async fn list(
    caller: Caller,
    State(state): State<ChatsState>,
    path: Result<Path<String>, PathRejection>,
    query: Result<Query<MessagesQuery>, QueryRejection>,
) -> Result<Response, ChatsError> {
    let Path(id) = path?;
    let id: ThreadId = id.parse()?;
    let Query(query) = query?;
    let before = query
        .before
        .as_deref()
        .filter(|text| !text.is_empty())
        .map(str::parse::<MessageId>)
        .transpose()?;
    let limit = page_size(query.limit, DEFAULT_PAGE, MAX_MESSAGES)?;
    let mut rows = state
        .store
        .list_messages(&caller.user, &id, before.as_ref(), limit + 1)
        .await?
        .ok_or(ChatsError::NotFound)?;
    let more = rows.len() > limit as usize;
    rows.truncate(limit as usize);
    let body = WireMessages {
        messages: rows.iter().map(message).collect(),
        more,
        before: rows
            .last()
            .filter(|_| more)
            .map(|row| row.id.as_str().to_owned()),
    };
    Ok(no_store(Json(body).into_response()))
}
