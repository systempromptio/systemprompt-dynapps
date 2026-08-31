//! `GET /api/public/atlassian/token` — the typed accessor core's external-MCP
//! client calls to obtain a fresh per-user Atlassian bearer.
//!
//! Serves the Jira and Confluence MCP servers alike: the response depends only
//! on who is calling, and core passes no server identity.

use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::Serialize;

use super::{AtlassianDeps, AtlassianError};
use crate::authz;
use crate::error::{AdminError, AdminResult};
use crate::handlers::users::extract_mcp_accessor_user;
use crate::repositories::users::atlassian_credentials;
use crate::services::atlassian_oauth;

#[derive(Debug, Serialize)]
struct TokenResponse {
    access_token: String,
    // Why: core deserializes only `access_token` and ignores the rest, so this
    // costs nothing today and saves a round trip if a direct-REST path ever
    // needs the site id.
    #[serde(skip_serializing_if = "Option::is_none")]
    cloud_id: Option<String>,
}

pub(crate) async fn atlassian_token_handler(
    Extension(deps): Extension<AtlassianDeps>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    let session = extract_mcp_accessor_user(&headers)?;
    if !deps.config.is_usable() {
        return Err(AdminError::Unavailable(
            "Atlassian not configured".to_owned(),
        ));
    }

    match atlassian_oauth::get_access_token(&deps.write_pool, &session.user_id, &deps.config).await
    {
        Ok(minted) => Ok(Json(TokenResponse {
            access_token: minted.access_token,
            cloud_id: minted.cloud_id,
        })
        .into_response()),

        // Why: core turns a 404 into "no token banked for this user; connect the
        // provider account first", which is exactly the instruction the user
        // needs. Any other status would hide it behind a generic failure.
        Err(AtlassianError::NotLinked) => Err(AdminError::NotFound(
            "Atlassian account not linked".to_owned(),
        )),

        // Why: the grant is dead and no retry will revive it. Drop the row so
        // the visibility gate closes and the user is prompted to reconnect,
        // rather than leaving them on a 502 that never clears.
        Err(AtlassianError::RefreshRejected) => {
            if let Err(e) =
                atlassian_credentials::delete_credential(&deps.write_pool, &session.user_id).await
            {
                tracing::warn!(error = %e, user_id = %session.user_id, "failed to clear rejected Atlassian credential");
            }
            authz::atlassian::invalidate(&session.user_id).await;
            tracing::info!(user_id = %session.user_id, "Atlassian refresh token rejected; credential cleared");
            Err(AdminError::NotFound(
                "Atlassian account not linked".to_owned(),
            ))
        },

        // Why: not `?` — a mint failure is an upstream fault. 502 says Atlassian
        // refused; a 500 would blame this server for Atlassian being down.
        // Why: lint-ok: error-adapt — deliberate 502 re-classification, see above
        Err(e) => Err(AdminError::Upstream(format!(
            "Atlassian token mint failed: {e}"
        ))),
    }
}
