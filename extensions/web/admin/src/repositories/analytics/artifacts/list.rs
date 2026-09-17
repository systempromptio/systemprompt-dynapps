//! The artifacts list and its KPI strip.

use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

use super::{ArtifactFilter, ArtifactRow, ArtifactStats};

pub async fn list_artifacts(
    pool: &PgPool,
    filter: &ArtifactFilter,
    limit: i64,
    offset: i64,
) -> Result<(Vec<ArtifactRow>, i64), sqlx::Error> {
    let rows = sqlx::query_as::<_, ArtifactRow>(include_str!("list.sql"))
        .bind(filter.since)
        .bind(filter.until)
        .bind(filter.user_id.as_ref().map(UserId::as_str))
        .bind(filter.subject_ids.as_deref())
        .bind(filter.session_id.as_ref().map(SessionId::as_str))
        .bind(filter.tool_name.as_deref())
        .bind(filter.source.as_deref())
        .bind(filter.structured_only)
        .bind(filter.errors_only)
        .bind(filter.skill.as_deref())
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;
    let total = rows.first().map_or(0, |r| r.total_rows);
    Ok((rows, total))
}

pub async fn get_artifact_stats(
    pool: &PgPool,
    filter: &ArtifactFilter,
) -> Result<ArtifactStats, sqlx::Error> {
    sqlx::query_as::<_, ArtifactStats>(include_str!("stats.sql"))
        .bind(filter.since)
        .bind(filter.until)
        .bind(filter.user_id.as_ref().map(UserId::as_str))
        .bind(filter.subject_ids.as_deref())
        .bind(filter.session_id.as_ref().map(SessionId::as_str))
        .bind(filter.tool_name.as_deref())
        .bind(filter.source.as_deref())
        .bind(filter.structured_only)
        .bind(filter.errors_only)
        .fetch_one(pool)
        .await
}
