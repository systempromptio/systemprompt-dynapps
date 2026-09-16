//! Client-reported session cost snapshots, listed per user.
//!
//! `session_cost_snapshots` holds one statusline-reported row per session,
//! keyed on `updated_at` (set-not-increment upserts) — so windows select by
//! last update and every figure is labeled "client-reported" on the page.

use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

#[derive(Debug, Clone)]
pub struct UserSessionCostRow {
    pub session_id: SessionId,
    pub model: Option<String>,
    pub total_cost_microdollars: i64,
    pub context_window_size: i64,
    pub cache_read_input_tokens: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_user_session_costs(
    pool: &PgPool,
    user_id: &UserId,
    limit: i64,
) -> Result<Vec<UserSessionCostRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT session_id AS "session_id!: SessionId", model,
               COALESCE(total_cost_microdollars, 0)::BIGINT AS "cost!",
               COALESCE(context_window_size, 0)::BIGINT AS "context!",
               COALESCE(cache_read_input_tokens, 0)::BIGINT AS "cache_read!",
               COALESCE(input_tokens, 0)::BIGINT AS "input!",
               COALESCE(output_tokens, 0)::BIGINT AS "output!",
               updated_at AS "updated_at!"
        FROM session_cost_snapshots
        WHERE user_id = $1
        ORDER BY updated_at DESC
        LIMIT $2
        "#,
        user_id.as_str(),
        limit,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| UserSessionCostRow {
            session_id: r.session_id,
            model: r.model,
            total_cost_microdollars: r.cost,
            context_window_size: r.context,
            cache_read_input_tokens: r.cache_read,
            input_tokens: r.input,
            output_tokens: r.output,
            updated_at: r.updated_at,
        })
        .collect())
}
