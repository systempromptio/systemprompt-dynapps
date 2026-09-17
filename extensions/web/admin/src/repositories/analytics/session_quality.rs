//! The human ratings people left on a session, read by
//! `/admin/sessions/{id}`. The judge's scores live in
//! `repositories::evaluation::runs`.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

/// One person's rating of one session, newest first.
#[derive(Debug, Clone)]
pub struct SessionRatingRow {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub rating: i16,
    pub outcome: String,
    pub notes: String,
    pub created_at: DateTime<Utc>,
}

pub async fn list_session_ratings(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Vec<SessionRatingRow>, sqlx::Error> {
    sqlx::query_as!(
        SessionRatingRow,
        r#"SELECT
               r.user_id      AS "user_id!: UserId",
               u.display_name AS "display_name?",
               r.rating       AS "rating!",
               r.outcome      AS "outcome!",
               r.notes        AS "notes!",
               r.created_at   AS "created_at!"
           FROM session_ratings r
           LEFT JOIN users u ON u.id = r.user_id
           WHERE r.session_id = $1
           ORDER BY r.created_at DESC
           LIMIT 50"#,
        session_id.as_str(),
    )
    .fetch_all(pool)
    .await
}
