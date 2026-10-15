//! Browser consent bound to the initiating login and connection generation,
//! and the bearer accessor core calls for a person or a service account.

use super::live_user;
use crate::repositories::connectors::{
    connector_accounts as accounts, connector_credentials as credentials,
};
use crate::services::connector_oauth::{self as oauth, Consent, Provider};
use crate::services::service_accounts::ServiceAccounts;
use crate::services::{connector_accounts as service, machine_credentials};
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::http::header::CACHE_CONTROL;
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Extension, Json};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_web_admin_core::auth::session::extract_mcp_accessor_user;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

// Why: core's external-MCP client forwards the run's declared purpose under
// this header; a service account's credential is spent only for the purposes
// `machine_credentials::caller` allows.
const CALLER_HEADER: &str = "x-systemprompt-caller";

#[derive(Deserialize)]
pub(crate) struct Start {
    resource_id: Option<String>,
    expected_user: Option<String>,
}

pub(crate) async fn start(
    State(pool): State<Arc<PgPool>>,
    Path(provider): Path<Provider>,
    headers: HeaderMap,
    Query(params): Query<Start>,
) -> AdminResult<Response> {
    let user = live_user(&pool, &headers, false).await?;
    if params
        .expected_user
        .is_some_and(|id| id != user.user_id.as_str())
    {
        return Err(AdminError::Forbidden(
            "Sign in to the same Systemprompt account as your bridge".into(),
        ));
    }
    service::require_entitlement(&pool, &user.user_id, provider.clone()).await?;
    let mut tx = pool.begin().await?;
    let account = accounts::get_locked_account(&mut tx, &user.user_id, provider.slug()).await?;
    tx.commit().await?;
    let state = consent_state(&provider);
    let verifier = URL_SAFE_NO_PAD.encode(rand::rng().random::<[u8; 32]>());
    let (url, mut grant) = oauth::authorize(
        Consent {
            user: user.user_id.as_str(),
            state: &state,
            verifier,
        },
        provider.clone(),
    )
    .await?;
    grant.session = user.session_id.map(|s| s.to_string());
    grant.generation = account.generation;
    grant.resource_id = params.resource_id.unwrap_or_default();
    credentials::save_state(
        &pool,
        &user.user_id,
        provider.slug(),
        &state,
        &oauth::seal(&grant)?,
    )
    .await?;
    Ok(([(CACHE_CONTROL, "no-store")], Redirect::to(&url)).into_response())
}

// Why: Salesforce orgs share one OAuth callback URL and the provider echoes
// `state` verbatim, so the server id prefixes it (`salesforce-uk.<random>`);
// only the random part binds the row.
fn consent_state(provider: &Provider) -> String {
    let random = URL_SAFE_NO_PAD.encode(rand::rng().random::<[u8; 32]>());
    if provider.is_salesforce() {
        format!("{}.{random}", provider.slug())
    } else {
        random
    }
}

#[derive(Deserialize)]
pub(crate) struct Callback {
    state: String,
    code: Option<String>,
    error: Option<String>,
    iss: Option<String>,
}

pub(crate) async fn callback(
    State(pool): State<Arc<PgPool>>,
    Path(provider): Path<Provider>,
    headers: HeaderMap,
    Query(params): Query<Callback>,
) -> AdminResult<Response> {
    complete(&pool, provider, &headers, params).await
}

pub(crate) async fn salesforce_callback(
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Query(params): Query<Callback>,
) -> AdminResult<Response> {
    let provider = params
        .state
        .split_once('.')
        .map(|(slug, _)| slug.to_owned())
        .and_then(|slug| Provider::try_from(slug).ok())
        .filter(Provider::is_salesforce)
        .ok_or_else(|| AdminError::BadRequest("Consent state names no Salesforce org".into()))?;
    complete(&pool, provider, &headers, params).await
}

async fn complete(
    pool: &PgPool,
    provider: Provider,
    headers: &HeaderMap,
    params: Callback,
) -> AdminResult<Response> {
    let user = live_user(pool, headers, false).await?;
    service::require_entitlement(pool, &user.user_id, provider.clone()).await?;
    let row = credentials::consume_state(pool, &user.user_id, provider.slug(), &params.state)
        .await?
        .ok_or_else(|| {
            AdminError::Unauthorized("Connector consent expired or already consumed".into())
        })?;
    let grant = oauth::open(&row, &user.user_id, &provider)?;
    if grant.session != user.session_id.as_ref().map(ToString::to_string) {
        return Err(AdminError::Unauthorized(
            "Login changed during connector consent".into(),
        ));
    }
    if !oauth::generic::authorization_response_issuer_matches(
        params.iss.as_deref(),
        &grant.authorization_issuer,
    ) {
        return Err(AdminError::Unauthorized(
            "Connector consent came back from a different authorization server".into(),
        ));
    }
    if params.error.is_some() {
        return Err(AdminError::Unauthorized("Connector consent denied".into()));
    }
    let code = params
        .code
        .ok_or_else(|| AdminError::BadRequest("Consent code missing".into()))?;
    super::completion::finish(pool, &user.user_id, headers, grant, &code).await
}

pub(crate) async fn token(
    State(pool): State<Arc<PgPool>>,
    Extension(accounts): Extension<ServiceAccounts>,
    Path(provider): Path<Provider>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    super::require_broker(&headers)?;
    // Why: core's MCP client presents a service account's in-process JWT, which
    // has no interactive session behind it; the purpose-bound caller check in
    // `broker_token` is that account's trust anchor instead of `live_user`.
    let claimed = extract_mcp_accessor_user(&headers)?;
    if accounts.is_service_account(&claimed.user_id).await? {
        let user = claimed;
        let caller = headers.get(CALLER_HEADER).and_then(|v| v.to_str().ok());
        let value =
            machine_credentials::broker_token(&pool, &user.user_id, provider.slug(), caller)
                .await?;
        return Ok(bearer_response(&value));
    }
    let user = live_user(&pool, &headers, false).await?;
    service::require_entitlement(&pool, &user.user_id, provider.clone()).await?;
    let access_token = oauth::verified_token(&pool, &user.user_id, provider.clone(), false).await?;
    Ok(bearer_response(&access_token))
}

// Why: Atlassian personal tokens use Basic auth, so the descriptor's scheme
// is empty and the trusted adapter supplies the complete header.
fn bearer_response(access_token: &str) -> Response {
    (
        [(CACHE_CONTROL, "no-store")],
        Json(serde_json::json!({"access_token": access_token})),
    )
        .into_response()
}
