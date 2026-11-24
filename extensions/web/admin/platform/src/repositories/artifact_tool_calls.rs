//! The artifacts broker's audit rows.
//!
//! One per tool call a signed-in viewer asked a Drive page to make: who, which
//! document, which server and tool, the outcome and how long it took. Never
//! the input or the result.

use sqlx::PgPool;
use systemprompt::identifiers::{DocumentId, McpServerId, McpToolName, UserId};

#[derive(Debug)]
pub struct NewToolCall<'a> {
    pub user: &'a UserId,
    pub document: &'a DocumentId,
    pub server: &'a McpServerId,
    pub tool: &'a McpToolName,
    pub outcome: &'a str,
    pub duration_ms: i32,
}

pub async fn record(pool: &PgPool, call: &NewToolCall<'_>) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO artifact_tool_calls
                (user_id, document_id, server, tool, outcome, duration_ms)
           VALUES ($1, $2::text::uuid, $3, $4, $5, $6)"#,
        call.user.as_str(),
        call.document.as_str(),
        call.server.as_str(),
        call.tool.as_str(),
        call.outcome,
        call.duration_ms,
    )
    .execute(pool)
    .await?;
    Ok(())
}
