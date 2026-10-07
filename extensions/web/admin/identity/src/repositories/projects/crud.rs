//! Project rows: read, create, rename, delete.
//!
//! `cost_centre` and `practice` are set and cleared through the same update:
//! an absent field keeps the stored value and an empty one clears it.
//!
//! No system row and no delete refusal: a project is work attribution, and
//! deleting one cascades its membership away, which is what an admin removing
//! a finished project means.

use sqlx::PgPool;
use systemprompt_web_shared::ProjectId;

use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::types::projects::{
    CreateProjectRequest, ProjectRow, UpdateProjectRequest,
};

pub async fn find_project(
    pool: &PgPool,
    project_id: &ProjectId,
) -> Result<Option<ProjectRow>, sqlx::Error> {
    sqlx::query_as!(
        ProjectRow,
        r#"SELECT id AS "id: ProjectId", name, description, source, cost_centre, practice FROM projects WHERE id = $1"#,
        project_id.as_str()
    )
    .fetch_optional(pool)
    .await
}

pub async fn insert_project(
    pool: &PgPool,
    req: &CreateProjectRequest,
    source: &str,
) -> AdminResult<ProjectRow> {
    if find_project(pool, &req.id).await?.is_some() {
        return Err(AdminError::Conflict(format!(
            "Project {} already exists",
            req.id
        )));
    }
    sqlx::query_as!(
        ProjectRow,
        r#"INSERT INTO projects (id, name, description, source, cost_centre, practice)
         VALUES ($1, $2, $3, $4, NULLIF(BTRIM($5), ''), NULLIF(BTRIM($6), ''))
         RETURNING id AS "id: ProjectId", name, description, source, cost_centre, practice"#,
        req.id.as_str(),
        req.name,
        req.description.as_deref(),
        source,
        req.cost_centre.as_deref(),
        req.practice.as_deref()
    )
    .fetch_one(pool)
    .await
    .map_err(AdminError::from)
}

pub async fn update_project(
    pool: &PgPool,
    project_id: &ProjectId,
    req: &UpdateProjectRequest,
) -> AdminResult<ProjectRow> {
    sqlx::query_as!(
        ProjectRow,
        r#"UPDATE projects SET name = COALESCE($2, name),
         description = COALESCE($3, description),
         cost_centre = CASE WHEN $4::text IS NULL THEN cost_centre ELSE NULLIF(BTRIM($4), '') END,
         practice = CASE WHEN $5::text IS NULL THEN practice ELSE NULLIF(BTRIM($5), '') END,
         updated_at = CURRENT_TIMESTAMP
         WHERE id = $1 RETURNING id AS "id: ProjectId", name, description, source, cost_centre, practice"#,
        project_id.as_str(),
        req.name.as_deref(),
        req.description.as_deref(),
        req.cost_centre.as_deref(),
        req.practice.as_deref()
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AdminError::NotFound(format!("Project {project_id} not found")))
}

pub async fn delete_project(pool: &PgPool, project_id: &ProjectId) -> AdminResult<()> {
    let deleted = sqlx::query!("DELETE FROM projects WHERE id = $1", project_id.as_str())
        .execute(pool)
        .await?;
    if deleted.rows_affected() == 0 {
        return Err(AdminError::NotFound(format!(
            "Project {project_id} not found"
        )));
    }
    Ok(())
}
