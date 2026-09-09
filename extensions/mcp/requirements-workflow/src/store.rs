//! Transactional workflow persistence and project authorization.
use crate::model::{Workflow, WorkflowId};
use anyhow::{Result, ensure};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

pub use crate::model::StoredWorkflow;

pub async fn migrate(pool: &PgPool) -> Result<()> {
    sqlx::raw_sql(include_str!("../schema/workflow.sql"))
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn require_project(pool: &PgPool, user: &UserId, project: &str) -> Result<()> {
    let allowed = sqlx::query_scalar!(
        r#"SELECT (EXISTS(SELECT 1 FROM users u WHERE u.id=$1 AND ('admin'=ANY(u.roles) OR 'platform_admin'=ANY(u.roles)))
            OR EXISTS(SELECT 1 FROM project_members WHERE user_id=$1 AND project_id=$2)
            OR ($2='storefront' AND EXISTS(SELECT 1 FROM group_members WHERE user_id=$1 AND group_id='europe-devs'))) AS "allowed!""#,
        user.as_str(),
        project
    )
    .fetch_one(pool)
    .await?;
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
    let id = workflow.id.0.to_string();
    sqlx::query!(
        "INSERT INTO requirements_workflows (id,owner_id,project_id,name,state) VALUES ($1,$2,$3,$4,$5)",
        id,
        user.as_str(),
        project,
        workflow.name,
        state
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "INSERT INTO requirements_workflow_events(workflow_id,revision,actor_id,event,state) VALUES ($1,0,$2,'created',$3)",
        id,
        user.as_str(),
        state
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(StoredWorkflow {
        revision: 0,
        owner_id: user.clone(),
        project_id: project.to_owned(),
        workflow,
    })
}

pub async fn get(pool: &PgPool, user: &UserId, id: WorkflowId) -> Result<StoredWorkflow> {
    let row = sqlx::query!(
        "SELECT revision,owner_id,project_id,state FROM requirements_workflows WHERE id=$1",
        id.0.to_string()
    )
    .fetch_one(pool)
    .await?;
    require_project(pool, user, &row.project_id).await?;
    ensure!(
        row.owner_id == user.as_str(),
        "Workflow evidence belongs to another source reader"
    );
    ensure!(
        row.state.get("validator_version").is_some(),
        "This OpenSpec workflow is archived. Download its evidence in the browser and start a new requirements workflow; old approvals cannot authorize new work."
    );
    Ok(StoredWorkflow {
        revision: row.revision,
        owner_id: UserId::new(row.owner_id),
        project_id: row.project_id,
        workflow: serde_json::from_value(row.state)?,
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
    let id = stored.workflow.id.0.to_string();
    let changed = sqlx::query!(
        "UPDATE requirements_workflows SET state=$1,revision=revision+1,updated_at=NOW() WHERE id=$2 AND revision=$3 AND owner_id=$4",
        state,
        id,
        old_revision,
        user.as_str()
    )
    .execute(&mut *tx)
    .await?;
    ensure!(
        changed.rows_affected() == 1,
        "Workflow changed; reload before retrying"
    );
    let revision = old_revision + 1;
    sqlx::query!(
        "INSERT INTO requirements_workflow_events(workflow_id,revision,actor_id,event,state) VALUES ($1,$2,$3,$4,$5)",
        id,
        revision,
        user.as_str(),
        event,
        state
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(revision)
}
