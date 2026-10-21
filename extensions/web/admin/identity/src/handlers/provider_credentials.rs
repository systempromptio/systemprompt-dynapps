//! The caller's own personal-subscription keys, served in-process.
//!
//! Core owns the per-person provider-credential store; these handlers call
//! its repository and key check directly with the session's `user_id`, so the
//! console needs no reachable `/v1` and no bearer token. Nothing here takes a
//! user from the request: there is no parameter that could point at somebody
//! else's key. The answers are shaped like core's `/v1/me/provider-credentials`
//! so the profile card and the bridge read the same rows.

use std::sync::Arc;

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::json;
use systemprompt::ai::repository::{UserProviderCredentialMeta, UserProviderCredentialRepository};
use systemprompt::ai::{
    PersonalKeyError, PersonalKeyRequest, personal_providers, store_personal_key,
};
use systemprompt::loader::ServicesBootstrap;
use systemprompt::manifest::services::{ProviderEntry, ProviderRegistry};
use systemprompt::models::bridge::profile::ProviderCredentialStatus;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::types::UserContext;

#[derive(Debug, Deserialize)]
pub(crate) struct SetKeyRequest {
    api_key: String,
    #[serde(default)]
    label: Option<String>,
}

fn status_row(
    entry: &ProviderEntry,
    meta: Option<&UserProviderCredentialMeta>,
) -> ProviderCredentialStatus {
    ProviderCredentialStatus {
        provider: entry.name.as_str().to_owned(),
        display_name: entry.display_name.clone(),
        configured: meta.is_some(),
        label: meta.and_then(|m| m.label.clone()),
        created_at: meta.map(|m| m.created_at),
        verified_at: meta.and_then(|m| m.verified_at),
        last_used_at: meta.and_then(|m| m.last_used_at),
    }
}

fn refusal(status: StatusCode, code: &str, message: &str) -> Response {
    (
        status,
        Json(json!({ "code": code, "error": code, "message": message })),
    )
        .into_response()
}

fn registry() -> AdminResult<ProviderRegistry> {
    let services = ServicesBootstrap::get().map_err(|e| AdminError::Internal(Box::new(e)))?;
    Ok(services.providers.clone())
}

fn personal_provider(registry: &ProviderRegistry, name: &str) -> Option<ProviderEntry> {
    personal_providers(registry)
        .into_iter()
        .find(|entry| entry.name.as_str() == name)
}

pub(crate) async fn list_own_provider_credentials(
    Extension(user_ctx): Extension<UserContext>,
    Extension(keys): Extension<Arc<UserProviderCredentialRepository>>,
) -> AdminResult<Response> {
    let stored = keys
        .list_metadata(&user_ctx.user_id)
        .await
        .map_err(|e| AdminError::Internal(Box::new(e)))?;
    let rows: Vec<ProviderCredentialStatus> = personal_providers(&registry()?)
        .iter()
        .map(|entry| {
            let meta = stored.iter().find(|m| m.provider == entry.name.as_str());
            status_row(entry, meta)
        })
        .collect();
    Ok(Json(rows).into_response())
}

pub(crate) async fn put_own_provider_credential(
    Extension(user_ctx): Extension<UserContext>,
    Extension(keys): Extension<Arc<UserProviderCredentialRepository>>,
    Path(provider): Path<String>,
    body: Result<Json<SetKeyRequest>, JsonRejection>,
) -> AdminResult<Response> {
    let Json(request) = body.map_err(|e| AdminError::BadRequest(e.body_text()))?;
    let registry = registry()?;
    let Some(entry) = personal_provider(&registry, &provider) else {
        return Ok(refusal(
            StatusCode::NOT_FOUND,
            "not_personal_provider",
            &format!("{provider} does not take a personal key."),
        ));
    };
    let label = request
        .label
        .as_deref()
        .map(str::trim)
        .filter(|l| !l.is_empty());
    let stored = store_personal_key(
        &keys,
        &registry,
        PersonalKeyRequest {
            user_id: &user_ctx.user_id,
            provider: entry.name.as_str(),
            key: request.api_key.trim(),
            label,
        },
    )
    .await;
    match stored {
        Ok(meta) => Ok(Json(status_row(&entry, Some(&meta))).into_response()),
        Err(PersonalKeyError::NotPersonalProvider(name)) => Ok(refusal(
            StatusCode::NOT_FOUND,
            "not_personal_provider",
            &format!("{name} does not take a personal key."),
        )),
        Err(PersonalKeyError::Rejected(name)) => Ok(refusal(
            StatusCode::UNPROCESSABLE_ENTITY,
            "personal_key_rejected",
            &format!("{name} refused this key."),
        )),
        Err(PersonalKeyError::Unverified { provider, status }) => Ok(refusal(
            StatusCode::BAD_REQUEST,
            "personal_key_unverified",
            &format!("{provider} could not verify the key ({status})."),
        )),
        Err(other) => Err(AdminError::Internal(Box::new(other))),
    }
}

pub(crate) async fn delete_own_provider_credential(
    Extension(user_ctx): Extension<UserContext>,
    Extension(keys): Extension<Arc<UserProviderCredentialRepository>>,
    Path(provider): Path<String>,
) -> AdminResult<Response> {
    if personal_provider(&registry()?, &provider).is_none() {
        return Ok(refusal(
            StatusCode::NOT_FOUND,
            "not_personal_provider",
            &format!("{provider} does not take a personal key."),
        ));
    }
    keys.delete_key(&user_ctx.user_id, &provider)
        .await
        .map_err(|e| AdminError::Internal(Box::new(e)))?;
    Ok(StatusCode::NO_CONTENT.into_response())
}
