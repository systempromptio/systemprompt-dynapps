//! The rows a run joins to outside the farm tables: the conversation context
//! its Claude Code session registered, and the gateway requests it made.

use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, UserId};

use crate::repositories::summaries::RunRequestRow;

pub async fn find_run_context_id(
    pool: &PgPool,
    user_id: &UserId,
    claude_session_id: &str,
) -> Result<Option<ContextId>, sqlx::Error> {
    sqlx::query_file_scalar!(
        "src/repositories/runs/find_run_context_id.sql",
        user_id.as_str(),
        claude_session_id,
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_run_requests(
    pool: &PgPool,
    gateway_session_id: &str,
) -> Result<Vec<RunRequestRow>, sqlx::Error> {
    sqlx::query_file_as!(
        RunRequestRow,
        "src/repositories/runs/list_run_requests.sql",
        gateway_session_id
    )
    .fetch_all(pool)
    .await
}
