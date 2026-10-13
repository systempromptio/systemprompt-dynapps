//! `GET /runs/{id}/events?after=&limit=`: the run's event stream from a
//! sequence number on, so a client follows a live run by passing back the
//! `next_after` it was given. Events are read from core's
//! `agent_run_events` through the run's `agent_run_id`.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use systemprompt::agent_runs::AgentRunRepository;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::types::UserContext;

use super::shared::load_visible_run;
use crate::state::FarmState;
use crate::types::FarmRunId;

const DEFAULT_LIMIT: i64 = 200;
const MAX_LIMIT: i64 = 1000;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct EventsQuery {
    after: Option<i64>,
    limit: Option<i64>,
}

// JSON: protocol boundary — `payload` is the stored `RunEventPayload`,
// re-encoded in the shape the JSONB column holds it.
#[derive(Debug, Serialize)]
struct EventJson {
    seq: i64,
    kind: String,
    tool_name: Option<String>,
    tool_use_id: Option<String>,
    payload: serde_json::Value,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct EventsResponse {
    events: Vec<EventJson>,
    next_after: i64,
}

pub(crate) async fn list_run_events(
    State(state): State<Arc<FarmState>>,
    Extension(ctx): Extension<UserContext>,
    Path(run_id): Path<FarmRunId>,
    Query(query): Query<EventsQuery>,
) -> AdminResult<Response> {
    let run = load_visible_run(&state.pool, &ctx, &run_id).await?;
    let after = query.after.unwrap_or(0).max(0);
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let rows = AgentRunRepository::from_pool(Arc::clone(&state.pool))
        .list_events_after(&run.agent_run_id, after, limit)
        .await
        .map_err(AdminError::internal)?;
    let next_after = rows.last().map_or(after, |r| r.seq);
    let events = rows
        .into_iter()
        .map(|r| {
            Ok(EventJson {
                seq: r.seq,
                kind: r.kind.as_str().to_owned(),
                tool_name: r.tool_name,
                tool_use_id: r.tool_use_id,
                payload: serde_json::to_value(&r.payload).map_err(AdminError::internal)?,
                created_at: r.created_at,
            })
        })
        .collect::<AdminResult<Vec<_>>>()?;
    Ok(Json(EventsResponse { events, next_after }).into_response())
}
