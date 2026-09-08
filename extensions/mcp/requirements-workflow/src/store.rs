//! Transactional workflow persistence and project authorization.
use crate::model::{Workflow, WorkflowId};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredWorkflow {
    pub revision: i64,
    pub owner_id: UserId,
    pub project_id: String,
    pub workflow: Workflow,
}

pub async fn migrate(pool: &PgPool) -> Result<()> {
    sqlx::raw_sql(include_str!("../schema/workflow.sql"))
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn require_project(pool: &PgPool, user: &UserId, project: &str) -> Result<()> {
    let allowed: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users u WHERE u.id=$1 AND ('admin'=ANY(u.roles) OR 'platform_admin'=ANY(u.roles))) OR EXISTS(SELECT 1 FROM project_members WHERE user_id=$1 AND project_id=$2) OR ($2='storefront' AND EXISTS(SELECT 1 FROM group_members WHERE user_id=$1 AND group_id='europe-devs'))"
    ).bind(user.as_str()).bind(project).fetch_one(pool).await?;
    ensure!(allowed, "Project access required");
    Ok(())
}

pub async fn create(
    pool: &PgPool,
    user: &UserId,
    project: &str,
    workflow: Workflow,
) -> Result<StoredWorkflow> {
    require_project(pool, user, project).await?;
    let mut tx = pool.begin().await?;
    let state = serde_json::to_value(&workflow)?;
    sqlx::query("INSERT INTO requirements_workflows (id,owner_id,project_id,name,state) VALUES ($1,$2,$3,$4,$5)")
        .bind(workflow.id.0.to_string()).bind(user.as_str()).bind(project).bind(&workflow.name).bind(&state).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO requirements_workflow_events(workflow_id,revision,actor_id,event,state) VALUES ($1,0,$2,'created',$3)")
        .bind(workflow.id.0.to_string()).bind(user.as_str()).bind(state).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(StoredWorkflow {
        revision: 0,
        owner_id: user.clone(),
        project_id: project.to_owned(),
        workflow,
    })
}

pub async fn get(pool: &PgPool, user: &UserId, id: WorkflowId) -> Result<StoredWorkflow> {
    let row = sqlx::query(
        "SELECT revision,owner_id,project_id,state FROM requirements_workflows WHERE id=$1",
    )
    .bind(id.0.to_string())
    .fetch_one(pool)
    .await?;
    let project: String = row.try_get("project_id")?;
    require_project(pool, user, &project).await?;
    let owner: String = row.try_get("owner_id")?;
    ensure!(
        owner == user.as_str(),
        "Workflow evidence belongs to another source reader"
    );
    Ok(StoredWorkflow {
        revision: row.try_get("revision")?,
        owner_id: UserId::new(owner),
        project_id: project,
        workflow: serde_json::from_value(row.try_get("state")?)?,
    })
}

pub async fn save(
    pool: &PgPool,
    user: &UserId,
    old_revision: i64,
    stored: &StoredWorkflow,
    event: &str,
) -> Result<i64> {
    require_project(pool, user, &stored.project_id).await?;
    let mut tx = pool.begin().await?;
    let state = serde_json::to_value(&stored.workflow)?;
    let changed=sqlx::query("UPDATE requirements_workflows SET state=$1,revision=revision+1,updated_at=NOW() WHERE id=$2 AND revision=$3 AND owner_id=$4")
        .bind(&state).bind(stored.workflow.id.0.to_string()).bind(old_revision).bind(user.as_str()).execute(&mut *tx).await?;
    ensure!(
        changed.rows_affected() == 1,
        "Workflow changed; reload before retrying"
    );
    let revision = old_revision + 1;
    sqlx::query("INSERT INTO requirements_workflow_events(workflow_id,revision,actor_id,event,state) VALUES ($1,$2,$3,$4,$5)")
        .bind(stored.workflow.id.0.to_string()).bind(revision).bind(user.as_str()).bind(event).bind(state).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(revision)
}
