//! Browser consent bound to the initiating login and connection generation,
//! and the bearer accessor core calls for a person or a service account.
//!
//! An admin may run the consent on behalf of a service account
//! (`?service_account=<id>`): the grant is stored for that account, and its
//! `client` / `service_account` mode farm runs then spend it. The state
//! carries the account (`sa.<id>.<random>`) so the callback knows whose
//! row to consume; the login check still binds it to the admin's session.

use super::live_user;
use crate::repositories::connectors::machine_credentials::find_machine_credential;
use crate::repositories::connectors::{
    connector_accounts as accounts, connector_credentials as credentials, farm_run_callers,
};
use crate::services::connector_oauth::{self as oauth, Consent, Provider};
use crate::services::service_accounts::ServiceAccounts;
use crate::services::{connector_accounts as service, machine_credentials};
use axum::extract::rejection::ExtensionRejection;
use axum::extract::{FromRequestParts, Path, Query, State};
use axum::http::HeaderMap;
use axum::http::header::CACHE_CONTROL;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Extension, Json};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::UserId;
use systemprompt::security::keys::SigningKeys;
use systemprompt_web_admin_core::auth::session::extract_mcp_accessor_user;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

// Why: core's external-MCP client forwards the run's declared purpose under
// this header; a service account's credential is spent only for the purposes
// `machine_credentials::caller` allows.
const CALLER_HEADER: &str = "x-systemprompt-caller";
const SERVICE_ACCOUNT_STATE: &str = "sa.";

#[derive(Debug, Clone)]
pub(crate) struct ConsentDeps {
    keys: SigningKeys,
    service_accounts: ServiceAccounts,
}

impl<S: Send + Sync> FromRequestParts<S> for ConsentDeps {
    type Rejection = ExtensionRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(keys) = Extension::from_request_parts(parts, state).await?;
        let Extension(service_accounts) = Extension::from_request_parts(parts, state).await?;
        Ok(Self {
            keys,
            service_accounts,
        })
    }
}

#[derive(Deserialize)]
pub(crate) struct Start {
    resource_id: Option<String>,
    expected_user: Option<String>,
    service_account: Option<UserId>,
}

pub(crate) async fn start(
    State(pool): State<Arc<PgPool>>,
    ConsentDeps {
        keys,
        service_accounts,
    }: ConsentDeps,
    Path(provider): Path<Provider>,
    headers: HeaderMap,
    Query(params): Query<Start>,
) -> AdminResult<Response> {
    let user = live_user(&pool, &keys, &headers, false).await?;
    if params
        .expected_user
        .is_some_and(|id| id != user.user_id.as_str())
    {
        return Err(AdminError::Forbidden(
            "Sign in to the same Systemprompt account as your bridge".into(),
        ));
    }
    let owner = if let Some(account) = params.service_account {
        require_acting_admin(&service_accounts, &user.user_id, &account).await?;
        account
    } else {
        service::require_entitlement(&pool, &user.user_id, provider.clone()).await?;
        user.user_id.clone()
    };
    let mut tx = pool.begin().await?;
    let account = accounts::get_locked_account(&mut tx, &owner, provider.slug()).await?;
    tx.commit().await?;
    let state = consent_state(&owner, &user.user_id);
    let verifier = URL_SAFE_NO_PAD.encode(rand::rng().random::<[u8; 32]>());
    let (url, mut grant) = oauth::authorize(
        Consent {
            user: owner.as_str(),
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
        &owner,
        provider.slug(),
        &state,
        &oauth::seal(&grant)?,
    )
    .await?;
    Ok(([(CACHE_CONTROL, "no-store")], Redirect::to(&url)).into_response())
}

fn consent_state(owner: &UserId, actor: &UserId) -> String {
    let random = URL_SAFE_NO_PAD.encode(rand::rng().random::<[u8; 32]>());
    if owner == actor {
        random
    } else {
        format!("{SERVICE_ACCOUNT_STATE}{owner}.{random}")
    }
}

fn service_account_of(state: &str) -> Option<UserId> {
    state
        .strip_prefix(SERVICE_ACCOUNT_STATE)
        .and_then(|rest| rest.rsplit_once('.'))
        .map(|(owner, _)| UserId::new(owner))
}

async fn require_acting_admin(
    service_accounts: &ServiceAccounts,
    actor: &UserId,
    account: &UserId,
) -> AdminResult<()> {
    if !service_accounts.is_admin(actor).await? {
        return Err(AdminError::Forbidden(
            "Only an admin connects a grant for a service account".into(),
        ));
    }
    if !service_accounts.is_service_account(account).await? {
        return Err(AdminError::NotFound("No such service account".into()));
    }
    Ok(())
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
    deps: ConsentDeps,
    Path(provider): Path<Provider>,
    headers: HeaderMap,
    Query(params): Query<Callback>,
) -> AdminResult<Response> {
    complete(&pool, &deps, provider, &headers, params).await
}

async fn complete(
    pool: &PgPool,
    ConsentDeps {
        keys,
        service_accounts,
    }: &ConsentDeps,
    provider: Provider,
    headers: &HeaderMap,
    params: Callback,
) -> AdminResult<Response> {
    let user = live_user(pool, keys, headers, false).await?;
    let owner = if let Some(account) = service_account_of(&params.state) {
        require_acting_admin(service_accounts, &user.user_id, &account).await?;
        account
    } else {
        service::require_entitlement(pool, &user.user_id, provider.clone()).await?;
        user.user_id.clone()
    };
    let row = credentials::consume_state(pool, &owner, provider.slug(), &params.state)
        .await?
        .ok_or_else(|| {
            AdminError::Unauthorized("Connector consent expired or already consumed".into())
        })?;
    let grant = oauth::open(&row, &owner, &provider)?;
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
    super::completion::finish(
        super::completion::CallbackSession {
            pool,
            keys,
            user: &owner,
            headers,
        },
        grant,
        &code,
    )
    .await
}

pub(crate) async fn token(
    State(pool): State<Arc<PgPool>>,
    Extension(keys): Extension<SigningKeys>,
    Extension(accounts): Extension<ServiceAccounts>,
    Path(provider): Path<Provider>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    super::require_broker(&headers)?;
    // Why: core's MCP client presents a service account's in-process JWT, which
    // has no interactive session behind it; the purpose-bound caller check in
    // `broker_token` is that account's trust anchor instead of `live_user`.
    let claimed = extract_mcp_accessor_user(&keys, &headers)?;
    let caller = headers.get(CALLER_HEADER).and_then(|v| v.to_str().ok());
    if accounts.is_service_account(&claimed.user_id).await? {
        let value =
            service_account_token(&pool, &accounts, &claimed.user_id, provider, caller).await?;
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
        service::require_entitlement(&pool, &claimed.user_id, provider.clone()).await?;
        let access_token = oauth::verified_token(&pool, &claimed.user_id, provider, false).await?;
        return Ok(bearer_response(&access_token));
    }
    let user = live_user(&pool, &keys, &headers, false).await?;
    service::require_entitlement(&pool, &user.user_id, provider.clone()).await?;
    let access_token = oauth::verified_token(&pool, &user.user_id, provider.clone(), false).await?;
    Ok(bearer_response(&access_token))
}

async fn service_account_token(
    pool: &PgPool,
    accounts: &ServiceAccounts,
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
    oauth::verified_token(pool, user, provider, false).await
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
