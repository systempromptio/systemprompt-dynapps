//! Session and activity tracking driven by webhook events.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;

use crate::error::{AdminError, AdminResult};
use crate::repositories::dashboard::usage_aggregations;
use crate::types::webhook::{StatusLinePayload, StatusLineQuery};

use super::helpers::authenticate_webhook;

const MICRODOLLARS_PER_DOLLAR: f64 = 1_000_000.0;

pub(crate) async fn track_statusline_event(
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Query(query): Query<StatusLineQuery>,
    Json(payload): Json<StatusLinePayload>,
) -> AdminResult<Response> {
    let user_id = authenticate_webhook(&headers)?;

    if let (Some(query_id), Some(payload_id)) = (&query.session_id, &payload.session_id)
        && query_id != payload_id
    {
        return Err(AdminError::BadRequest(
            "Query and payload session_id must match".to_owned(),
        ));
    }
    let session_id = query
        .session_id
        .as_ref()
        .or(payload.session_id.as_ref())
        .ok_or_else(|| AdminError::BadRequest("session_id is required".to_owned()))?;
    crate::types::webhook::validate_session_key(session_id.as_str())
        .map_err(AdminError::BadRequest)?;
    {
        let model = payload.model.as_ref().and_then(|m| m.api_model_id.clone());
        if payload
            .cost
            .and_then(|c| c.total_cost_usd)
            .is_some_and(|v| {
                !v.is_finite() || v < 0.0 || v > i64::MAX as f64 / MICRODOLLARS_PER_DOLLAR
            })
        {
            return Err(AdminError::BadRequest("Invalid cost".to_owned()));
        }
        let cost = payload
            .cost
            .and_then(|c| c.total_cost_usd)
            .map(usd_to_microdollars);
        let window = payload.context_window;
        let usage = window.and_then(|w| w.current_usage);
        if window
            .and_then(|w| w.context_window_size)
            .is_some_and(|v| v < 0)
            || usage.is_some_and(|u| {
                [
                    u.input,
                    u.output,
                    u.cache_creation_input,
                    u.cache_read_input,
                ]
                .into_iter()
                .flatten()
                .any(|v| v < 0)
            })
        {
            return Err(AdminError::BadRequest(
                "Token counts must be nonnegative".to_owned(),
            ));
        }

        let snapshot = usage_aggregations::SessionCostSnapshot {
            session_id,
            user_id: &user_id,
            model: model.as_deref(),
            total_cost_microdollars: cost,
            context_window_size: window.and_then(|w| w.context_window_size),
            input_tokens: usage.and_then(|u| u.input),
            output_tokens: usage.and_then(|u| u.output),
            cache_creation_input_tokens: usage.and_then(|u| u.cache_creation_input),
            cache_read_input_tokens: usage.and_then(|u| u.cache_read_input),
        };
        usage_aggregations::upsert_session_cost_snapshot(&pool, &snapshot).await?;
        usage_aggregations::set_session_summary_tokens(&pool, &snapshot).await?;
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}

fn usd_to_microdollars(usd: f64) -> i64 {
    let micro = usd * MICRODOLLARS_PER_DOLLAR;
    if micro.is_finite() {
        // Why: round-to-nearest keeps sub-cent costs; clamp guards absurd inputs.
        micro.round().clamp(0.0, i64::MAX as f64) as i64
    } else {
        0
    }
}
