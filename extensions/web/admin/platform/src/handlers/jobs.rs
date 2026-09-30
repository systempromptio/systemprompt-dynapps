//! HTTP handlers for inspecting and triggering scheduled jobs.

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;

use crate::services::jobs_service;
use systemprompt_web_admin_core::error::AdminResult;

use systemprompt_web_admin_core::types::responses::JobsListResponse;

pub(crate) async fn list_jobs_handler(State(pool): State<Arc<PgPool>>) -> AdminResult<Response> {
    let jobs = jobs_service::list_jobs(&pool).await?;
    Ok(Json(JobsListResponse { jobs }).into_response())
}
