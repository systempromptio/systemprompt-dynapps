//! One user's latest conversations — the profile pane's recent list and the
//! roster's "what were they last doing" link. Side-call-only conversations are
//! never listed here: a probe is not something a person talked about.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, SessionId, UserId};

use super::ConversationRow;

pub async fn find_latest_conversation(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Option<ConversationRow>, sqlx::Error> {
    Ok(list_recent_conversations(pool, user_id, 1).await?.pop())
}

pub async fn list_recent_conversations(
    pool: &PgPool,
    user_id: &UserId,
    limit: i64,
) -> Result<Vec<ConversationRow>, sqlx::Error> {
    sqlx::query_as!(
        ConversationRow,
        r#"
        SELECT
            r.context_id                AS "context_id!: ContextId",
            r.title                     AS "title!",
            r.user_id                   AS "user_id?: UserId",
            r.display_name              AS "display_name?",
            r.session_id                AS "session_id?: SessionId",
            r.client_session_id         AS "client_session_id?",
            r.group_name                AS "group_name?",
            r.project_name              AS "project_name?",
            r.model                     AS "model?",
            r.turn_count                AS "turn_count!",
            r.side_call_count           AS "side_call_count!",
            r.side_call_cost_microdollars AS "side_call_cost_microdollars!",
            r.tool_call_count           AS "tool_call_count!",
            r.error_count               AS "error_count!",
            r.total_input_tokens        AS "total_input_tokens!",
            r.total_output_tokens       AS "total_output_tokens!",
            r.total_cost_microdollars   AS "total_cost_microdollars!",
            r.first_at                  AS "first_at?",
            r.last_at                   AS "last_at?",
            r.status                    AS "status?"
        FROM conversation_rollups r
        WHERE r.user_id = $1 AND r.turn_count > 0
        ORDER BY r.last_at DESC NULLS LAST, r.context_id
        LIMIT $2
        "#,
        user_id.as_str(),
        limit,
    )
    .fetch_all(pool)
    .await
}
