//! HTTP handlers for gateway route configuration.
//!
//! Settings (`enabled`, `auth_scheme`, `inference_path_prefix`) still live
//! in the file alone; routes are read from and written to the
//! `gateway_routes` table, and every route write regenerates the file's
//! `routes:` sequence so the next restart dispatches what was saved.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;

use crate::repositories;
use crate::repositories::gateway_routes::editor;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::request as shared;
use systemprompt_web_admin_core::types::{
    GatewayConfigView, GatewayRouteView, ReorderRoutesRequest, UpdateGatewaySettingsRequest,
};

#[derive(Debug, Serialize)]
pub(crate) struct CreateRouteResponse {
    pub index: usize,
}

pub(crate) async fn gateway_config_view(pool: &PgPool) -> AdminResult<GatewayConfigView> {
    let gateway_path = shared::get_gateway_file_path()?;
    let mut config =
        systemprompt_web_admin_core::repositories::gateway_catalog::get_gateway_config(
            &gateway_path,
        )
        .map_err(AdminError::internal)?;
    config.routes = repositories::gateway_routes::rows::list_gateway_routes(pool)
        .await?
        .into_iter()
        .map(|r| r.route)
        .collect();
    Ok(config)
}

pub(crate) async fn get_gateway_handler(State(pool): State<Arc<PgPool>>) -> AdminResult<Response> {
    Ok(Json(gateway_config_view(&pool).await?).into_response())
}

pub(crate) async fn update_gateway_settings_handler(
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<UpdateGatewaySettingsRequest>,
) -> AdminResult<Response> {
    let gateway_path = shared::get_gateway_file_path()?;
    systemprompt_web_admin_core::repositories::gateway_catalog::update_gateway_settings(
        &gateway_path,
        &body,
    )?;
    Ok(Json(gateway_config_view(&pool).await?).into_response())
}

pub(crate) async fn create_gateway_route_handler(
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<GatewayRouteView>,
) -> AdminResult<Response> {
    let gateway_path = shared::get_gateway_file_path()?;
    let index = editor::create_route_entry(&pool, &gateway_path, &body).await?;
    Ok((StatusCode::CREATED, Json(CreateRouteResponse { index })).into_response())
}

pub(crate) async fn update_gateway_route_handler(
    State(pool): State<Arc<PgPool>>,
    Path(idx): Path<usize>,
    Json(body): Json<GatewayRouteView>,
) -> AdminResult<Response> {
    let gateway_path = shared::get_gateway_file_path()?;
    editor::update_route_at(&pool, &gateway_path, idx, &body).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

pub(crate) async fn delete_gateway_route_handler(
    State(pool): State<Arc<PgPool>>,
    Path(idx): Path<usize>,
) -> AdminResult<Response> {
    let gateway_path = shared::get_gateway_file_path()?;
    editor::delete_route_at(&pool, &gateway_path, idx).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

pub(crate) async fn reorder_gateway_routes_handler(
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<ReorderRoutesRequest>,
) -> AdminResult<Response> {
    let gateway_path = shared::get_gateway_file_path()?;
    editor::reorder_route_positions(&pool, &gateway_path, &body.order).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}
