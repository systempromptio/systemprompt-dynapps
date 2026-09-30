//! Group rows: read, create, rename, delete.
//!
//! `unassigned` is a real row rather than a special case in every query, so
//! deleting it is refused here as well as by the database trigger: the repo
//! answers with a 409 a handler can return, the trigger is the backstop for
//! anything that bypasses this path. Member counts read the `user_groups`
//! view, so derived `unassigned` members count like everyone else.

use sqlx::PgPool;
use systemprompt_web_shared::GroupId;

use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::types::groups::{
    CreateGroupRequest, GroupRecord, UpdateGroupRequest,
};

pub async fn find_group(
    pool: &PgPool,
    group_id: &GroupId,
) -> Result<Option<GroupRecord>, sqlx::Error> {
    sqlx::query_as!(
        GroupRecord,
        r#"SELECT id AS "id: GroupId", name, description, is_system, source FROM groups WHERE id = $1"#,
        group_id.as_str()
    )
    .fetch_optional(pool)
    .await
}

pub async fn insert_group(
    pool: &PgPool,
    req: &CreateGroupRequest,
    source: &str,
) -> AdminResult<GroupRecord> {
    if find_group(pool, &req.id).await?.is_some() {
        return Err(AdminError::Conflict(format!(
            "Group {} already exists",
            req.id
        )));
    }
    sqlx::query_as!(
        GroupRecord,
        r#"INSERT INTO groups (id, name, description, source) VALUES ($1, $2, $3, $4)
         RETURNING id AS "id: GroupId", name, description, is_system, source"#,
        req.id.as_str(),
        req.name,
        req.description.as_deref(),
        source
    )
    .fetch_one(pool)
    .await
    .map_err(AdminError::from)
}

pub async fn update_group(
    pool: &PgPool,
    group_id: &GroupId,
    req: &UpdateGroupRequest,
) -> AdminResult<GroupRecord> {
    sqlx::query_as!(
        GroupRecord,
        r#"UPDATE groups SET name = COALESCE($2, name), description = COALESCE($3, description),
         updated_at = CURRENT_TIMESTAMP WHERE id = $1
         RETURNING id AS "id: GroupId", name, description, is_system, source"#,
        group_id.as_str(),
        req.name.as_deref(),
        req.description.as_deref()
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AdminError::NotFound(format!("Group {group_id} not found")))
}

pub async fn delete_group(pool: &PgPool, group_id: &GroupId) -> AdminResult<()> {
    let group = find_group(pool, group_id)
        .await?
        .ok_or_else(|| AdminError::NotFound(format!("Group {group_id} not found")))?;
    if group.is_system {
        return Err(AdminError::Conflict(format!(
            "Group {group_id} is a system group and cannot be deleted"
        )));
    }
    sqlx::query!("DELETE FROM groups WHERE id = $1", group_id.as_str())
        .execute(pool)
        .await?;
    Ok(())
}
