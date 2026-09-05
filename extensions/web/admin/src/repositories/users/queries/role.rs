//! The access profile a single user carries: their roles, their groups and
//! their projects, read in one round trip.
//!
//! Every request resolves this once, so it is one query rather than three:
//! the middleware needs all of it to build a `UserContext`, and the callers
//! that only want roles pay nothing extra for the arrays.

use sqlx::{FromRow, PgPool};
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone, FromRow)]
pub struct UserAccessProfile {
    pub roles: Vec<String>,
    pub group_ids: Vec<String>,
    pub project_ids: Vec<String>,
}

pub async fn find_user_access_profile(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Option<UserAccessProfile>, sqlx::Error> {
    sqlx::query_as::<_, UserAccessProfile>(
        r"
        SELECT
            u.roles AS roles,
            COALESCE(
                (SELECT ARRAY_AGG(ug.group_id ORDER BY ug.group_id)
                 FROM user_groups ug WHERE ug.user_id = u.id),
                ARRAY[]::TEXT[]
            ) AS group_ids,
            COALESCE(
                (SELECT ARRAY_AGG(DISTINCT pm.project_id)
                 FROM project_members pm WHERE pm.user_id = u.id),
                ARRAY[]::TEXT[]
            ) AS project_ids
        FROM users u
        WHERE u.id = $1
        ",
    )
    .bind(user_id.as_str())
    .fetch_optional(pool)
    .await
}
