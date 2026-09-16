//! `POST /sync/sources/refresh` — re-resolve the pinned sources through
//! core's own refresh endpoint.
//!
//! Core owns the fetch-verify-compose pipeline and the restart handle; an
//! extension has neither (`ExtensionContext` exposes no restart). So this
//! handler proxies to `POST /api/v1/admin/services/refresh?restart=true` on
//! the local listener, forwarding the caller's own token — the console
//! cookie is a core API JWT — so the same administrator authorises both hops
//! and no token is minted. Core's response is returned verbatim.

use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::extract::{Extension, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::config::ProfileBootstrap;

use crate::activity::{self, NewActivity};
use crate::error::{AdminError, AdminResult};
use crate::handlers::users::extract_token_from_headers;
use crate::types::UserContext;

// Why: a refresh downloads and verifies every pinned bundle; two minutes is
// generous for a handful of kits and short enough that a wedged registry
// does not hold the console's connection forever.
const REFRESH_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CoreRefreshResponse {
    pub changed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composed_hash: Option<String>,
    // JSON: core's per-source views, relayed as-is
    #[serde(default)]
    pub sources: Vec<serde_json::Value>,
    pub restarting: bool,
}

pub(crate) async fn refresh_sources_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    let token = extract_token_from_headers(&headers)?;
    let profile = ProfileBootstrap::get()?;
    let url = format!(
        "http://127.0.0.1:{}/api/v1/admin/services/refresh?restart=true",
        profile.server.port
    );
    let client = reqwest::Client::builder()
        .timeout(REFRESH_TIMEOUT)
        .build()
        .map_err(AdminError::internal)?;
    let resp = client
        .post(&url)
        .bearer_auth(token)
        .send()
        .await
        // Why: reqwest has no variant of its own; an unreachable local
        // listener is a 502 by design. lint-ok: error-adapt
        .map_err(|e| AdminError::Upstream(format!("services refresh unreachable: {e}")))?;
    let status = resp.status();
    // Why: the body only decorates the error message; one that cannot be read
    // still reports the status it came with.
    // Why: discard-ok: the status is the answer, the body is decoration
    let body = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(match status.as_u16() {
            409 => AdminError::Conflict("a services refresh is already running".to_owned()),
            401 | 403 => AdminError::Forbidden(
                "the services refresh needs an instance administrator".to_owned(),
            ),
            _ => AdminError::Upstream(format!("services refresh returned {status}: {body}")),
        });
    }
    let parsed: CoreRefreshResponse = serde_json::from_str(&body)
        // Why: a reply core produced that does not parse is an upstream fault
        // by design. lint-ok: error-adapt
        .map_err(|e| AdminError::Upstream(format!("services refresh reply unreadable: {e}")))?;
    activity::record(
        &pool,
        NewActivity::sources_refreshed(&user_ctx.user_id, parsed.changed, parsed.restarting),
    )
    .await;
    Ok(Json(parsed).into_response())
}
