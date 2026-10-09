//! `POST /govern/authz` — the installation's authz decision as an HTTP
//! endpoint, for the MCP server and external callers that cannot link the
//! in-process hook.
//!
//! The gateway itself uses [`crate::authz_hook`] (`mode: extension`); both
//! answer with [`crate::authz_eval::evaluate`], so the decision, its caches
//! and its audit rows are identical whichever way the question arrives.

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;
use systemprompt_security::authz::AuthzRequest;

// Why: core's hook caller reads a non-200 status as "hook unavailable" and
// lets the call through, so every answer is 200, a failure a deny body.
pub(crate) async fn govern_authz(
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<AuthzRequest>,
) -> Response {
    let decision = crate::authz_eval::evaluate(&pool, req).await;
    (StatusCode::OK, Json(decision)).into_response()
}
