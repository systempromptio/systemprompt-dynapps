//! The rows a run joins to outside the farm tables: the conversation context
//! its host session registered, and the gateway requests it made. Both keys
//! come from the run's `agent_runs` row.

use sqlx::PgPool;
use systemprompt::identifiers::{ApiKeyId, ContextId, HostSessionId, SessionId, UserId};

use crate::repositories::summaries::RunRequestRow;

pub async fn find_run_context_id(
    pool: &PgPool,
    user_id: &UserId,
    host_session_id: &HostSessionId,
) -> Result<Option<ContextId>, sqlx::Error> {
    sqlx::query_file_scalar!(
        "src/repositories/runs/find_run_context_id.sql",
        user_id.as_str(),
        host_session_id.as_str(),
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_run_requests(
    pool: &PgPool,
    gateway_session_id: &SessionId,
) -> Result<Vec<RunRequestRow>, sqlx::Error> {
    sqlx::query_file_as!(
        RunRequestRow,
        "src/repositories/runs/list_run_requests.sql",
        gateway_session_id.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn count_run_requests(pool: &PgPool, api_key_id: &ApiKeyId) -> Result<i64, sqlx::Error> {
    sqlx::query_file_scalar!(
        "src/repositories/runs/count_run_requests.sql",
        api_key_id.as_str()
    )
    .fetch_one(pool)
    .await
}
