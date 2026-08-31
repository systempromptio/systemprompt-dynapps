//! `POST /admin/api/profile/atlassian/unlink` — drop the caller's Atlassian
//! credential.

use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use std::sync::Arc;

use crate::authz;
use crate::error::AdminResult;
use crate::repositories::users::atlassian_credentials;
use crate::types::UserContext;

pub(crate) async fn atlassian_unlink(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<sqlx::PgPool>>,
) -> AdminResult<Response> {
    // Why: an absent credential is not an error — the caller asked for it gone
    // and it is gone. Reporting 404 would make a double-click look like a bug.
    let unlinked = atlassian_credentials::delete_credential(&pool, &user_ctx.user_id).await?;
    authz::atlassian::invalidate(&user_ctx.user_id).await;
    tracing::info!(user_id = %user_ctx.user_id, unlinked, "Atlassian account unlinked");
    Ok(Json(serde_json::json!({ "unlinked": unlinked })).into_response())
}
