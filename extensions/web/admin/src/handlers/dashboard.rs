//! The dashboard counters and the activity events feed.
//!
//! They answer the admin API's `GET /dashboard` and `GET /events`.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;

use crate::repositories;
use systemprompt_web_admin_core::error::AdminResult;
use systemprompt_web_admin_core::types::EventsQuery;

pub(crate) async fn dashboard_handler(State(pool): State<Arc<PgPool>>) -> AdminResult<Response> {
    let data =
        repositories::dashboard::get_dashboard_data(&pool, "7 days", "4 hours", "today", "7d")
            .await?;
    Ok(Json(data).into_response())
}

pub(crate) async fn list_events_handler(
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<EventsQuery>,
) -> AdminResult<Response> {
    let response = repositories::dashboard::get_events_page(&pool, &query).await?;
    Ok(Json(response).into_response())
}
