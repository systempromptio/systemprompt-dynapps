//! Role and project lookup for a single user.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::types::Project;

pub async fn find_user_roles_project(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Option<(Vec<String>, Option<Project>)>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT u.roles, upe.project
        FROM users u
        LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
        WHERE u.id = $1
        "#,
        user_id.as_str()
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| (r.roles, Project::parse_opt(r.project.as_deref()))))
}
