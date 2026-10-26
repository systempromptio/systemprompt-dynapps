//! Browser consent bound to the initiating login and connection generation.
//!
//! An admin may run the consent on behalf of a service account
//! (`?service_account=<id>`): the grant is stored for that account, and its
//! `client` / `service_account` mode farm runs then spend it. The state
//! carries the account (`sa.<id>.<random>`) so the callback knows whose
//! row to consume; the login check still binds it to the admin's session.

use super::deps::ConsentDeps;
use super::live_user;
use crate::repositories::connectors::{
    connector_accounts as accounts, connector_credentials as credentials,
};
use crate::services::connector_accounts as service;
use crate::services::connector_oauth::{self as oauth, Consent, Provider};
use crate::services::service_accounts::ServiceAccounts;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::http::header::CACHE_CONTROL;
use axum::response::{IntoResponse, Redirect, Response};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

const SERVICE_ACCOUNT_STATE: &str = "sa.";

#[derive(Deserialize)]
pub(crate) struct Start {
    resource_id: Option<String>,
    expected_user: Option<String>,
    service_account: Option<UserId>,
}

pub(crate) async fn start(
    State(pool): State<Arc<PgPool>>,
    ConsentDeps {
        boot,
        service_accounts,
        services,
    }: ConsentDeps,
    Path(provider): Path<Provider>,
    headers: HeaderMap,
    Query(params): Query<Start>,
) -> AdminResult<Response> {
    let user = live_user(&pool, &boot, &headers, false).await?;
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
        service::require_entitlement(&services, &pool, &user.user_id, provider.clone()).await?;
        user.user_id.clone()
    };
    let mut tx = pool.begin().await?;
    let account = accounts::get_locked_account(&mut tx, &owner, provider.slug()).await?;
    tx.commit().await?;
    let state = consent_state(&owner, &user.user_id);
    let verifier = URL_SAFE_NO_PAD.encode(rand::rng().random::<[u8; 32]>());
    let (url, mut grant) = oauth::authorize(
        &services,
        &boot,
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
        &oauth::seal(boot.secrets().as_ref(), &grant)?,
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
        boot,
        service_accounts,
        services,
    }: &ConsentDeps,
    provider: Provider,
    headers: &HeaderMap,
    params: Callback,
) -> AdminResult<Response> {
    let user = live_user(pool, boot, headers, false).await?;
    let owner = if let Some(account) = service_account_of(&params.state) {
        require_acting_admin(service_accounts, &user.user_id, &account).await?;
        account
    } else {
        service::require_entitlement(services, pool, &user.user_id, provider.clone()).await?;
        user.user_id.clone()
    };
    let row = credentials::consume_state(pool, &owner, provider.slug(), &params.state)
        .await?
        .ok_or_else(|| {
            AdminError::Unauthorized("Connector consent expired or already consumed".into())
        })?;
    let grant = oauth::open(services, boot.secrets().as_ref(), &row, &owner, &provider)?;
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
        services,
        super::completion::CallbackSession {
            pool,
            boot,
            user: &owner,
            headers,
        },
        grant,
        &code,
    )
    .await
}
