//! `GET /api/public/salesforce/token` — the typed accessor core's external-MCP
//! client calls to obtain a fresh per-user Salesforce bearer.

use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

use super::{SalesforceDeps, SalesforceError, salesforce_private_key};
use crate::repositories::connectors::salesforce_identity;
use crate::services::salesforce_jwt_bearer;
use systemprompt_web_admin_core::auth::session::extract_mcp_accessor_user;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

#[derive(Debug, Deserialize)]
pub(crate) struct SalesforceTokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub instance_url: Option<String>,
}

pub(crate) async fn post_token_request(
    token_url: &str,
    body: String,
) -> Result<SalesforceTokenResponse, SalesforceError> {
    let resp = reqwest::Client::new()
        .post(token_url)
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(body)
        .send()
        .await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp
            .text()
            .await
            .inspect_err(|e| tracing::warn!(error = %e, "salesforce token: error body unreadable"))
            .unwrap_or_default();
        return Err(SalesforceError::TokenEndpoint { status, body });
    }
    Ok(resp.json().await?)
}

#[derive(Debug, Serialize)]
struct TokenResponse {
    access_token: String,
    // Why: core reads only `access_token`, but Salesforce Hosted MCP is
    // addressed per-instance, so the instance URL must travel with it.
    instance_url: String,
}

pub(crate) async fn salesforce_token_handler(
    Extension(deps): Extension<SalesforceDeps>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    let session = extract_mcp_accessor_user(&headers)?;
    if !deps.config.is_usable() {
        return Err(AdminError::Unavailable(
            "Salesforce not configured".to_owned(),
        ));
    }

    // Why: core's external-MCP client turns a 404 into "connect the provider
    // account first"; minting with no linked `sub` would fail opaquely.
    let Some(username) =
        salesforce_identity::find_username(&deps.write_pool, &session.user_id, "salesforce")
            .await?
    else {
        return Err(AdminError::NotFound(
            "Salesforce account not linked".to_owned(),
        ));
    };

    let private_key = salesforce_private_key().ok_or_else(|| {
        AdminError::Unavailable("Salesforce signing key not configured".to_owned())
    })?;
    let fresh = salesforce_jwt_bearer::get_token(&deps.config, &username, &private_key)
        .await
        .map_err(|source| AdminError::upstream("Salesforce token mint failed", source))?;

    Ok(Json(TokenResponse {
        access_token: fresh.access_token,
        instance_url: fresh.instance_url,
    })
    .into_response())
}
