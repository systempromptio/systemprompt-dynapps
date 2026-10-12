//! `GET /agent-farm/farms` and `GET /agent-farm/farms/{id}`: every farm with
//! its sync state, and one farm with the agents its last sync projected.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

use crate::repositories::rows::{AgentRow, FarmRow};
use crate::repositories::{agents, farms};
use crate::types::FarmId;

#[derive(Debug, Serialize)]
struct FarmsResponse {
    farms: Vec<FarmRow>,
}

#[derive(Debug, Serialize)]
struct FarmDetailResponse {
    farm: FarmRow,
    agents: Vec<AgentRow>,
}

pub(crate) async fn list_farms(State(pool): State<Arc<PgPool>>) -> AdminResult<Response> {
    let farms = farms::list_farms(&pool).await?;
    Ok(Json(FarmsResponse { farms }).into_response())
}

pub(crate) async fn get_farm(
    State(pool): State<Arc<PgPool>>,
    Path(farm_id): Path<FarmId>,
) -> AdminResult<Response> {
    let farm = farms::find_farm(&pool, &farm_id)
        .await?
        .ok_or_else(|| AdminError::NotFound(format!("farm {}", farm_id.as_str())))?;
    let agents = agents::list_farm_agents(&pool, &farm_id).await?;
    Ok(Json(FarmDetailResponse { farm, agents }).into_response())
}
