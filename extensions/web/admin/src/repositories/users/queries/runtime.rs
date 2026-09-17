//! Live bridge runtime detail for one user, for the user-detail page.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct UserRuntimeDetail {
    pub connected_agents: i64,
    pub total_agents: i64,
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub last_bridge_version: Option<String>,
    pub last_os: Option<String>,
    pub last_hostname: Option<String>,
    pub last_heartbeat_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn get_user_runtime_detail(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<UserRuntimeDetail, sqlx::Error> {
    let totals = sqlx::query!(
        r#"
        SELECT
            COALESCE(COUNT(*) FILTER (WHERE last_heartbeat_at > now() - interval '5 minutes'), 0)::BIGINT AS "connected_agents!",
            COALESCE(SUM(tokens_in_total), 0)::BIGINT AS "tokens_in!",
            COALESCE(SUM(tokens_out_total), 0)::BIGINT AS "tokens_out!"
        FROM bridge_sessions WHERE user_id = $1
        "#,
        user_id.as_str()
    )
    .fetch_one(pool)
    .await?;

    let total_agents: i64 = 0;

    let latest = sqlx::query!(
        r#"
        SELECT bridge_version, os, hostname, last_heartbeat_at
        FROM bridge_sessions
        WHERE user_id = $1
        ORDER BY last_heartbeat_at DESC
        LIMIT 1
        "#,
        user_id.as_str()
    )
    .fetch_optional(pool)
    .await?;

    Ok(UserRuntimeDetail {
        connected_agents: totals.connected_agents,
        total_agents,
        tokens_in: totals.tokens_in,
        tokens_out: totals.tokens_out,
        last_bridge_version: latest.as_ref().map(|r| r.bridge_version.clone()),
        last_os: latest.as_ref().map(|r| r.os.clone()),
        last_hostname: latest.as_ref().map(|r| r.hostname.clone()),
        last_heartbeat_at: latest.map(|r| r.last_heartbeat_at),
    })
}
