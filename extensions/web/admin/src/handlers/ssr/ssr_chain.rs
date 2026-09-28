//! `GET /admin/api/chain/:id` — the chain-of-custody envelope as JSON.
//!
//! `:id` may be a `decision_id`, `request_id`, `trace_id`, or `session_id`; the
//! repository resolves it to a `session_id` and returns the full chain of
//! custody (decisions, `ai_requests`, plugin events, transcript, summary).

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;

use crate::repositories::governance::chain::find_decision_chain;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::types::UserContext;

pub(crate) async fn chain_envelope(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> AdminResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }

    let envelope = find_decision_chain(&pool, &id)
        .await?
        .ok_or_else(|| AdminError::NotFound(format!("No audit chain for id {id}")))?;

    Ok(Json(envelope).into_response())
}
