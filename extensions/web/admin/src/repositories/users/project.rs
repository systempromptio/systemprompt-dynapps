//! Storage for the project AD FS placed a user in. Written on every SSO
//! sign-in (replace, not merge) so the directory, not a stale row, decides.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::types::Project;

pub async fn set_project(
    pool: &PgPool,
    user_id: &UserId,
    project: Option<Project>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO user_profile_ext (user_id, project) VALUES ($1, $2) \
         ON CONFLICT (user_id) DO UPDATE SET project = EXCLUDED.project",
        user_id.as_str(),
        project.map(Project::as_str),
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find_project(pool: &PgPool, user_id: &UserId) -> Result<Option<Project>, sqlx::Error> {
    let row = sqlx::query_scalar!(
        "SELECT project FROM user_profile_ext WHERE user_id = $1",
        user_id.as_str()
    )
    .fetch_optional(pool)
    .await?;
    Ok(Project::parse_opt(row.flatten().as_deref()))
}
