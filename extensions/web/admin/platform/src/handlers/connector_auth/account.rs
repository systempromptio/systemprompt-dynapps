//! Account state and explicit mutations shared by the browser and bridge.

use super::live_user;
use crate::repositories::connectors::{
    connector_accounts as accounts, connector_credentials as credentials,
};
use crate::services::connector_accounts as service;
use crate::services::connector_oauth::{self as oauth, Provider};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::http::header::CACHE_CONTROL;
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_web_admin_core::connections::source::get_connections;
use systemprompt_web_admin_core::error::AdminResult;

pub(crate) async fn list(
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    let user = live_user(&pool, &headers, false).await?;
    Ok((
        [(CACHE_CONTROL, "no-store")],
        Json(get_connections(&pool, &user.user_id).await?),
    )
        .into_response())
}

pub(crate) async fn disconnect(
    State(pool): State<Arc<PgPool>>,
    Path(provider): Path<Provider>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    let user = live_user(&pool, &headers, true).await?;
    let mut tx = pool.begin().await?;
    let mut row = accounts::get_locked_account(&mut tx, &user.user_id, provider.slug()).await?;
    credentials::delete(&mut tx, &user.user_id, provider.slug()).await?;
    accounts::delete_pending(&mut tx, &user.user_id, provider.slug()).await?;
    row.status = "not_connected".into();
    row.auth_method = None;
    row.account_id = None;
    row.account_name = None;
    row.resource_id = None;
    row.resource_name = None;
    row.error_code = None;
    row.verified_at = None;
    row.generation += 1;
    accounts::update_account(&mut tx, &user.user_id, &row).await?;
    tx.commit().await?;
    tracing::info!(user_id = %user.user_id, provider = provider.slug(), "connector_disconnected");
    Ok((
        [(CACHE_CONTROL, "no-store")],
        Json(get_connections(&pool, &user.user_id).await?),
    )
        .into_response())
}

pub(crate) async fn test(
    State(pool): State<Arc<PgPool>>,
    Path(provider): Path<Provider>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    let user = live_user(&pool, &headers, true).await?;
    service::require_entitlement(&pool, &user.user_id, provider.clone()).await?;
    let report = if provider.is_session_attested() {
        service::probe_session_connection(&pool, &user.user_id, &provider).await?
    } else {
        oauth::probe_connection(&pool, &user.user_id, provider.clone()).await?
    };
    tracing::info!(user_id = %user.user_id, provider = provider.slug(), ok = report.ok, "connector_probed");
    let mut snapshot = get_connections(&pool, &user.user_id).await?;
    snapshot.verification = Some(report);
    Ok(([(CACHE_CONTROL, "no-store")], Json(snapshot)).into_response())
}

pub(super) fn apply_verified(row: &mut accounts::ProviderConnection, grant: &oauth::Grant) {
    row.status = "connected".into();
    row.auth_method = Some(grant.auth_method.clone());
    row.account_id = Some(grant.account_id.clone());
    row.account_name = Some(grant.account_name.clone());
    row.resource_id = Some(grant.resource_id.clone());
    row.resource_name = Some(grant.resource_name.clone());
    row.error_code = None;
    row.verified_at = Some(Utc::now());
}
