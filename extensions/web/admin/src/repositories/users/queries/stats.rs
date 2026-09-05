//! Per-project user counts and activity, for the access-control overview.

use sqlx::PgPool;

use crate::types::ProjectStats;

pub async fn list_project_stats(pool: &PgPool) -> Result<Vec<ProjectStats>, sqlx::Error> {
    sqlx::query_as!(
        ProjectStats,
        r#"
        SELECT
            COALESCE(upe.project, 'unassigned') AS "project!",
            COUNT(DISTINCT u.id)::BIGINT AS "user_count!",
            COUNT(DISTINCT u.id) FILTER (WHERE u.status = 'active')::BIGINT AS "active_count!",
            COALESCE(SUM(ev.event_count), 0)::BIGINT AS "total_events!",
            COUNT(DISTINCT u.id) FILTER (
                WHERE ev.last_event >= NOW() - INTERVAL '24 hours'
            )::BIGINT AS "active_24h!"
        FROM users u
        LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
        LEFT JOIN (
            SELECT user_id, COUNT(*)::BIGINT AS event_count, MAX(created_at) AS last_event
            FROM plugin_usage_events GROUP BY user_id
        ) ev ON ev.user_id = u.id
        WHERE NOT ('anonymous' = ANY(u.roles))
        GROUP BY 1
        ORDER BY 2 DESC, 1
        "#,
    )
    .fetch_all(pool)
    .await
}
