//! One-shot approvals of a live page's write: the viewer's Allow for one
//! call, bound to the viewer, the document, the server, the tool and the hash
//! of the input, and spent by the call that presents it.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{DocumentId, UserId};

#[derive(Debug)]
pub struct ArtifactApproval<'a> {
    pub id: &'a str,
    pub user: &'a UserId,
    pub document: &'a DocumentId,
    pub server: &'a str,
    pub tool: &'a str,
    pub input_hash: &'a str,
}

pub async fn insert_artifact_approval(
    pool: &PgPool,
    approval: &ArtifactApproval<'_>,
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"WITH expired AS (
               DELETE FROM artifact_approvals WHERE expires_at <= CURRENT_TIMESTAMP
           )
           INSERT INTO artifact_approvals
                (id, user_id, document_id, server, tool, input_hash, expires_at)
           VALUES ($1, $2, $3::text::uuid, $4, $5, $6, $7)"#,
        approval.id,
        approval.user.as_str(),
        approval.document.as_str(),
        approval.server,
        approval.tool,
        approval.input_hash,
        expires_at,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_artifact_approval(
    pool: &PgPool,
    approval: &ArtifactApproval<'_>,
) -> Result<bool, sqlx::Error> {
    let spent = sqlx::query!(
        r#"DELETE FROM artifact_approvals
            WHERE id = $1 AND user_id = $2 AND document_id = $3::text::uuid
              AND server = $4 AND tool = $5 AND input_hash = $6
              AND expires_at > CURRENT_TIMESTAMP"#,
        approval.id,
        approval.user.as_str(),
        approval.document.as_str(),
        approval.server,
        approval.tool,
        approval.input_hash,
    )
    .execute(pool)
    .await?;
    Ok(spent.rows_affected() == 1)
}
