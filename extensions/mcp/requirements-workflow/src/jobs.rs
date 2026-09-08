//! Persisted validation jobs keep model latency outside the MCP request.
use crate::model::Phase;
use crate::store::{self, StoredWorkflow};
use crate::validation;
use anyhow::Result;
use serde_json::{Value, json};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use uuid::Uuid;

pub async fn list(pool: &PgPool, stored: &StoredWorkflow) -> Result<Vec<Value>> {
    Ok(sqlx::query_scalar("SELECT to_jsonb(j) || CASE WHEN status='running' AND started_at < NOW()-INTERVAL '16 minutes' THEN jsonb_build_object('status','failed','error','Validation worker expired; retry validation') ELSE '{}'::jsonb END FROM requirements_validation_jobs j WHERE workflow_id=$1 ORDER BY started_at DESC LIMIT 20")
        .bind(stored.workflow.id.0.to_string()).fetch_all(pool).await?)
}

pub async fn start(
    pool: PgPool,
    user: UserId,
    stored: StoredWorkflow,
    phase: Phase,
) -> Result<Value> {
    stored.workflow.require_predecessors(phase)?;
    anyhow::ensure!(
        stored.workflow.artifacts.contains_key(&phase),
        "Submit the artifact first"
    );
    let mut tx = pool.begin().await?;
    let revision: i64 =
        sqlx::query_scalar("SELECT revision FROM requirements_workflows WHERE id=$1 FOR UPDATE")
            .bind(stored.workflow.id.0.to_string())
            .fetch_one(&mut *tx)
            .await?;
    anyhow::ensure!(
        revision == stored.revision,
        "Workflow changed; reload before validating"
    );
    let existing: Option<String> = sqlx::query_scalar("SELECT id FROM requirements_validation_jobs WHERE workflow_id=$1 AND phase=$2 AND revision=$3 AND status='running' AND started_at>NOW()-INTERVAL '16 minutes' ORDER BY started_at DESC LIMIT 1")
        .bind(stored.workflow.id.0.to_string()).bind(phase.name()).bind(stored.revision).fetch_optional(&mut *tx).await?;
    if let Some(id) = existing {
        return Ok(json!({"job_id":id,"status":"running"}));
    }
    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO requirements_validation_jobs(id,workflow_id,phase,revision,status) VALUES($1,$2,$3,$4,'running')")
        .bind(&id).bind(stored.workflow.id.0.to_string()).bind(phase.name()).bind(stored.revision).execute(&mut *tx).await?;
    tx.commit().await?;
    let job_id = id.clone();
    tokio::spawn(async move {
        let mut stored = stored;
        let result = validation::validate(&stored.workflow, phase).await;
        let (status, error) = match result {
            Ok(result) => {
                if let Some(artifact) = stored.workflow.artifacts.get_mut(&phase) {
                    artifact.validation = Some(result);
                }
                match store::save(
                    &pool,
                    &user,
                    stored.revision,
                    &stored,
                    "validation_completed",
                )
                .await
                {
                    Ok(_) => ("complete", None),
                    Err(_) => (
                        "stale",
                        Some("Workflow changed during validation; reload and retry".to_owned()),
                    ),
                }
            },
            Err(error) => ("failed", Some(error.to_string())),
        };
        if let Err(error) = sqlx::query("UPDATE requirements_validation_jobs SET status=$2,error=$3,finished_at=NOW() WHERE id=$1")
            .bind(&job_id).bind(status).bind(error).execute(&pool).await {
            tracing::error!(job_id, %error, "Could not record validation completion");
        }
    });
    Ok(
        json!({"job_id":id,"status":"running","message":"Poll get_requirements_workflow; only its saved validation result unlocks the next phase"}),
    )
}
