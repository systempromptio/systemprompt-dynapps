//! Display names for the user ids a drive page shows: document owners,
//! revision authors, grant subjects and live editors.

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{Email, UserId};

#[derive(Debug, Serialize)]
pub struct DrivePerson {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub email: Option<Email>,
}

pub async fn list_drive_people(
    pool: &PgPool,
    user_ids: &[String],
) -> Result<Vec<DrivePerson>, sqlx::Error> {
    sqlx::query_as!(
        DrivePerson,
        r#"SELECT
                u.id AS "user_id!: UserId",
                COALESCE(u.display_name, u.full_name, u.name) AS display_name,
                u.email AS "email?: Email"
            FROM users u
            WHERE u.id = ANY($1)"#,
        user_ids,
    )
    .fetch_all(pool)
    .await
}
