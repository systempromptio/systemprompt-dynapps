//! Documents — the document store's revisions attributed to a skill, a
//! conversation or the whole window.
//!
//! They are read from core's `report_document_revisions` view. A revision is
//! the unit of work: a document counts as created when the attributed
//! revision is its first.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, DocumentId};

/// One document a skill or a conversation created or edited.
#[derive(Debug, Clone)]
pub struct DocumentTouchRow {
    pub document_id: DocumentId,
    pub name: String,
    pub format: String,
    pub created: bool,
    pub revisions: i64,
    pub last_at: DateTime<Utc>,
}

pub const DOCUMENTS_LISTED: i64 = 50;

pub async fn list_skill_documents(
    pool: &PgPool,
    window: &super::skills::SkillWindow,
    skill: &str,
) -> Result<Vec<DocumentTouchRow>, sqlx::Error> {
    let rows = sqlx::query_file!(
        "src/repositories/analysis/documents/skill.sql",
        window.start,
        window.end,
        skill,
        window.subject_ids.as_deref(),
        DOCUMENTS_LISTED,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| DocumentTouchRow {
            document_id: DocumentId::new(r.document_id),
            name: r.name,
            format: r.format,
            created: r.created,
            revisions: r.revisions,
            last_at: r.last_at,
        })
        .collect())
}

pub async fn list_conversation_documents(
    pool: &PgPool,
    context_id: &ContextId,
) -> Result<Vec<DocumentTouchRow>, sqlx::Error> {
    let rows = sqlx::query_file!(
        "src/repositories/analysis/documents/conversation.sql",
        context_id.as_str(),
        DOCUMENTS_LISTED,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| DocumentTouchRow {
            document_id: DocumentId::new(r.document_id),
            name: r.name,
            format: r.format,
            created: r.created,
            revisions: r.revisions,
            last_at: r.last_at,
        })
        .collect())
}

pub async fn count_documents_touched(
    pool: &PgPool,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    subject_ids: Option<&[String]>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_file_scalar!(
        "src/repositories/analysis/documents/touched.sql",
        start,
        end,
        subject_ids,
    )
    .fetch_one(pool)
    .await
}
