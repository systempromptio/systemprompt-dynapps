//! Turning a [`ScopeRequest`] into the concrete user id list a scoped listing
//! binds.
//!
//! One query answers it for every listing: membership lives in `user_groups`
//! and `project_members`, and resolving it once here means a listing joins
//! whatever it likes and adds a single `user_id = ANY($n)` rather than
//! restating what membership means.

use sqlx::PgPool;

use crate::util::scope::{ScopeRequest, SubjectScope};

pub async fn get_subject_scope(
    pool: &PgPool,
    request: &ScopeRequest,
) -> Result<SubjectScope, sqlx::Error> {
    let groups = request.group_filter();
    if groups.is_none() && request.project.is_none() {
        return Ok(SubjectScope::All);
    }

    let ids: Vec<String> = sqlx::query_scalar::<_, String>(
        r"
        SELECT u.id
        FROM users u
        WHERE ($1::TEXT[] IS NULL OR EXISTS (
                  SELECT 1 FROM user_groups ug
                  WHERE ug.user_id = u.id AND ug.group_id = ANY($1)))
          AND ($2::TEXT IS NULL OR EXISTS (
                  SELECT 1 FROM project_members pm
                  WHERE pm.user_id = u.id AND pm.project_id = $2))
        ",
    )
    .bind(groups)
    .bind(request.project.as_deref())
    .fetch_all(pool)
    .await?;

    Ok(SubjectScope::Users(ids))
}
