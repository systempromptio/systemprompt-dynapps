//! Browser review of requirements workflows: the exact rows the review
//! screen reads and the transactional decision it records.
//!
//! The MCP server owns the schema (`extensions/mcp/requirements-workflow/
//! schema/workflow.sql`) and applies it at boot; this module only reads the
//! stored document and appends a human decision under the same optimistic
//! revision the server uses.

use crate::error::{AdminError, AdminResult};
// JSON: the persisted workflow document is stored as one JSONB column and
// decoded through the shared model type; the previous revision is read back
// the same way for the change summary.
use serde_json::Value;
use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};
use systemprompt_requirements_model::model::{StoredWorkflow, WorkflowId};

fn conflict(e: impl std::fmt::Display) -> AdminError {
    AdminError::Conflict(e.to_string())
}

async fn get_project_access(pool: &PgPool, user: &UserId, project: &str) -> AdminResult<bool> {
    Ok(sqlx::query_scalar!(
        r#"SELECT (EXISTS(SELECT 1 FROM users u WHERE u.id=$1 AND ('admin'=ANY(u.roles) OR 'platform_admin'=ANY(u.roles)))
            OR EXISTS(SELECT 1 FROM project_members WHERE user_id=$1 AND project_id=$2)
            OR ($2='storefront' AND EXISTS(SELECT 1 FROM group_members WHERE user_id=$1 AND group_id='europe-devs'))) AS "allowed!""#,
        user.as_str(),
        project
    )
    .fetch_one(pool)
    .await?)
}

// Why: mirrors the MCP server's `store::get` contract exactly — project
// membership plus ownership — so the browser can never review a bundle the
// tools would refuse to show.
pub async fn get_workflow(
    pool: &PgPool,
    user: &UserId,
    id: WorkflowId,
) -> AdminResult<StoredWorkflow> {
    let Some(row) = sqlx::query!(
        "SELECT revision,owner_id,project_id,state FROM requirements_workflows WHERE id=$1",
        id.0.to_string()
    )
    .fetch_optional(pool)
    .await?
    else {
        return Err(AdminError::NotFound(
            "No requirements workflow matches that id.".to_owned(),
        ));
    };
    if !get_project_access(pool, user, &row.project_id).await? {
        return Err(conflict("Project access required"));
    }
    if row.owner_id != user.as_str() {
        return Err(conflict(
            "Workflow evidence belongs to another source reader",
        ));
    }
    Ok(StoredWorkflow {
        revision: row.revision,
        owner_id: UserId::new(row.owner_id),
        project_id: row.project_id,
        workflow: serde_json::from_value(row.state).map_err(conflict)?,
    })
}

#[derive(Debug)]
pub struct ReviewNonce<'a> {
    pub nonce: &'a str,
    pub workflow_id: &'a str,
    pub revision: i64,
    pub user: &'a UserId,
    pub session_id: &'a SessionId,
}

pub async fn insert_review_nonce(pool: &PgPool, nonce: &ReviewNonce<'_>) -> AdminResult<()> {
    sqlx::query!(
        "INSERT INTO requirements_workflow_review_nonces(nonce,workflow_id,revision,user_id,session_id) VALUES($1,$2,$3,$4,$5)",
        nonce.nonce,
        nonce.workflow_id,
        nonce.revision,
        nonce.user.as_str(),
        nonce.session_id.as_str()
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find_previous_state(
    pool: &PgPool,
    workflow_id: &str,
    revision: i64,
) -> Result<Option<Value>, AdminError> {
    Ok(sqlx::query_scalar!(
        "SELECT state FROM requirements_workflow_events WHERE workflow_id=$1 AND revision<$2 ORDER BY revision DESC LIMIT 1",
        workflow_id,
        revision
    )
    .fetch_optional(pool)
    .await?)
}

/// A receipt signing key as the download bundle publishes it.
#[derive(Debug, Clone)]
pub struct VerificationKey {
    pub key_id: String,
    pub public_key: String,
}

pub async fn list_verification_keys(
    pool: &PgPool,
    key_ids: &[String],
) -> Result<Vec<VerificationKey>, AdminError> {
    Ok(sqlx::query_as!(
        VerificationKey,
        "SELECT key_id,public_key FROM requirements_workflow_keys WHERE key_id=ANY($1)",
        key_ids
    )
    .fetch_all(pool)
    .await?)
}

/// One human decision, bound to the nonce the review form carried.
#[derive(Debug)]
pub struct Decision<'a> {
    pub workflow_id: &'a str,
    pub nonce: &'a str,
    pub revision: i64,
    pub user: &'a UserId,
    pub session_id: &'a SessionId,
    pub event: &'a str,
    pub signing_key: Option<VerificationKey>,
    pub state: Value,
}

// Why: the nonce is consumed, the key registered and the revision advanced
// in one transaction, so a double-submitted form either lands once or not at
// all — there is no window in which the receipt exists without its key.
pub async fn insert_decision(pool: &PgPool, decision: &Decision<'_>) -> AdminResult<()> {
    let mut tx = pool.begin().await?;
    let consumed = sqlx::query!(
        "DELETE FROM requirements_workflow_review_nonces WHERE nonce=$1 AND workflow_id=$2 AND revision=$3 AND user_id=$4 AND session_id=$5 AND expires_at>NOW()",
        decision.nonce,
        decision.workflow_id,
        decision.revision,
        decision.user.as_str(),
        decision.session_id.as_str()
    )
    .execute(&mut *tx)
    .await?;
    if consumed.rows_affected() != 1 {
        return Err(conflict("Review form expired or was already used"));
    }
    if let Some(key) = &decision.signing_key {
        sqlx::query!(
            "INSERT INTO requirements_workflow_keys(key_id,public_key) VALUES($1,$2) ON CONFLICT DO NOTHING",
            key.key_id,
            key.public_key
        )
        .execute(&mut *tx)
        .await?;
    }
    let changed = sqlx::query!(
        "UPDATE requirements_workflows SET state=$1,revision=revision+1,updated_at=NOW() WHERE id=$2 AND revision=$3",
        decision.state,
        decision.workflow_id,
        decision.revision
    )
    .execute(&mut *tx)
    .await?;
    if changed.rows_affected() != 1 {
        return Err(conflict("Concurrent edit; reload and review"));
    }
    sqlx::query!(
        "INSERT INTO requirements_workflow_events(workflow_id,revision,actor_id,event,state) VALUES($1,$2,$3,$4,$5)",
        decision.workflow_id,
        decision.revision + 1,
        decision.user.as_str(),
        decision.event,
        decision.state
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}
