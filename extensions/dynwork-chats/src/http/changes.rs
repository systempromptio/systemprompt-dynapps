//! `GET /changes?since=<cursor>&limit=<n>`.
//!
//! The threads changed after the cursor, oldest change first, tombstones
//! included, and the cursor to ask with next. Without `since` it is the full
//! list of live threads. `limit` defaults to 200 and is capped at the store's
//! page.

use axum::Json;
use axum::extract::rejection::QueryRejection;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::auth::Caller;
use crate::error::ChatsError;
use crate::http::params::page_size;
use crate::http::response::no_store;
use crate::http::wire::{WireChanges, WireThread};
use crate::ids::Cursor;
use crate::state::ChatsState;
use crate::store::MAX_PAGE;

pub const DEFAULT_PAGE: u32 = 200;

/// The query of the changes feed.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ChangesQuery {
    pub since: Option<String>,
    pub limit: Option<u32>,
}

pub async fn list(
    caller: Caller,
    State(state): State<ChatsState>,
    query: Result<Query<ChangesQuery>, QueryRejection>,
) -> Result<Response, ChatsError> {
    let Query(query) = query?;
    let since = query
        .since
        .as_deref()
        .filter(|text| !text.is_empty())
        .map(str::parse::<Cursor>)
        .transpose()?;
    let limit = page_size(query.limit, DEFAULT_PAGE, MAX_PAGE)?;
    let page = state
        .store
        .changes_since(&caller.user, since, limit)
        .await?;
    let body = WireChanges {
        threads: page.threads.iter().map(WireThread::from).collect(),
        cursor: page.cursor.to_string(),
        more: page.has_more,
    };
    Ok(no_store(Json(body).into_response()))
}
