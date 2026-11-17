//! `GET /changes?since=<cursor>&limit=<n>`: the caller's routines and
//! tombstones changed after the cursor, oldest first, and the cursor to send
//! next. An absent `since` starts from the beginning.

use axum::Json;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use serde::Deserialize;

use crate::error::RoutinesError;
use crate::state::RoutinesState;
use crate::store;
use crate::view::{ChangesPage, RoutineView};

const DEFAULT_LIMIT: i64 = 200;
const MAX_LIMIT: i64 = 500;

// Why: both are read as text so a malformed one answers after the token check.
#[derive(Debug, Deserialize)]
pub struct ChangesQuery {
    since: Option<String>,
    limit: Option<String>,
}

pub async fn list(
    State(state): State<RoutinesState>,
    headers: HeaderMap,
    Query(query): Query<ChangesQuery>,
) -> Result<Json<ChangesPage>, RoutinesError> {
    let desktop = super::caller(&state, &headers)?;
    let since = parse(
        query.since.as_deref(),
        0,
        "since is a cursor from this feed",
    )?;
    let limit =
        parse(query.limit.as_deref(), DEFAULT_LIMIT, "limit is a number")?.clamp(1, MAX_LIMIT);
    let mut records = store::list_changes(&state.pool, &desktop.user, since, limit + 1).await?;
    let more = records.len() > usize::try_from(limit).unwrap_or(usize::MAX);
    records.truncate(usize::try_from(limit).unwrap_or(usize::MAX));
    let cursor = records.last().map_or(since, |record| record.change_seq);
    Ok(Json(ChangesPage {
        routines: records.into_iter().map(RoutineView::from).collect(),
        cursor: cursor.to_string(),
        more,
    }))
}

fn parse(raw: Option<&str>, default: i64, problem: &'static str) -> Result<i64, RoutinesError> {
    raw.map_or(Ok(default), |text| {
        text.parse::<i64>()
            .ok()
            .filter(|value| *value >= 0)
            .ok_or(RoutinesError::BadRequest(problem))
    })
}
