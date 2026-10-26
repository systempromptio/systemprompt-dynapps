//! The bearer accessor core calls for a person or a service account: it
//! resolves the caller from the forwarded token and returns a live access
//! token for the requested provider, spending a service account's credential
//! only for the purposes its caller header allows.

use super::deps::ConsentDeps;
use super::live_user;
use crate::repositories::connectors::farm_run_callers;
use crate::repositories::connectors::machine_credentials::find_machine_credential;
use crate::services::connector_oauth::{self as oauth, Provider};
use crate::services::{connector_accounts as service, machine_credentials};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::http::header::CACHE_CONTROL;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::auth::session::extract_mcp_accessor_user;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

// Why: core's external-MCP client forwards the run's declared purpose under
// this header; a service account's credential is spent only for the purposes
// `machine_credentials::caller` allows.
const CALLER_HEADER: &str = "x-systemprompt-caller";

pub(crate) async fn token(
    State(pool): State<Arc<PgPool>>,
    deps: ConsentDeps,
    Path(provider): Path<Provider>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    let ConsentDeps {
        boot,
        service_accounts: accounts,
        services,
    } = &deps;
    super::require_broker(boot.secrets().as_ref(), &headers)?;
    // Why: core's MCP client presents a service account's in-process JWT, which
    // has no interactive session behind it; the purpose-bound caller check in
    // `broker_token` is that account's trust anchor instead of `live_user`.
    let claimed = extract_mcp_accessor_user(boot.token_authority().as_ref(), &headers)?;
    let caller = headers.get(CALLER_HEADER).and_then(|v| v.to_str().ok());
    if accounts.is_service_account(&claimed.user_id).await? {
        let value = service_account_token(&pool, &deps, &claimed.user_id, provider, caller).await?;
        return Ok(bearer_response(&value));
    }
    // Why: core runs a `run_as: {mode: user}` farm run headless, so its
    // person's bearer has no live login; the active run its signed purpose
    // names is the trust anchor instead, and entitlement is still checked.
    if let Some(run) = machine_credentials::caller::farm_run(caller) {
        if !farm_run_callers::get_farm_run_active(&pool, run, &claimed.user_id).await? {
            return Err(AdminError::Forbidden(
                "The farm run this bearer names is not active for its user".into(),
            ));
        }
        service::require_entitlement(services, &pool, &claimed.user_id, provider.clone()).await?;
        let access_token = oauth::verified_token(
            oauth::ConnectorAccess {
                services,
                secrets: boot.secrets().as_ref(),
            },
            &pool,
            &claimed.user_id,
            provider,
            false,
        )
        .await?;
        return Ok(bearer_response(&access_token));
    }
    let user = live_user(&pool, boot, &headers, false).await?;
    service::require_entitlement(services, &pool, &user.user_id, provider.clone()).await?;
    let access_token = oauth::verified_token(
        oauth::ConnectorAccess {
            services,
            secrets: boot.secrets().as_ref(),
        },
        &pool,
        &user.user_id,
        provider.clone(),
        false,
    )
    .await?;
    Ok(bearer_response(&access_token))
}

async fn service_account_token(
    pool: &PgPool,
    ConsentDeps {
        boot,
        service_accounts: accounts,
        services,
    }: &ConsentDeps,
    user: &UserId,
    provider: Provider,
    caller: Option<&str>,
) -> AdminResult<String> {
    let holds_machine = find_machine_credential(pool, user, provider.slug())
        .await?
        .is_some();
    if holds_machine || !machine_credentials::caller::allowed(caller) {
        return machine_credentials::broker_token(pool, accounts, user, provider.slug(), caller)
            .await;
    }
    oauth::verified_token(
        oauth::ConnectorAccess {
            services,
            secrets: boot.secrets().as_ref(),
        },
        pool,
        user,
        provider,
        false,
    )
    .await
}

// Why: core's machine grants mint the complete Authorization header, so the
// descriptor's scheme is empty and the trusted adapter supplies it whole.
fn bearer_response(access_token: &str) -> Response {
    (
        [(CACHE_CONTROL, "no-store")],
        Json(serde_json::json!({"access_token": access_token})),
    )
        .into_response()
}
