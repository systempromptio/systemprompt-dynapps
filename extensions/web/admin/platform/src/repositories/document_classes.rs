//! The class rows on folders and documents, and the lineage a class is
//! resolved from.
//!
//! Folder ancestry is core's `document_folders.ancestor_ids` (root first);
//! the creating agent is core's `documents.created_by_agent`.

use sqlx::PgPool;
use uuid::Uuid;

pub const KIND_FOLDER: &str = "folder";
pub const KIND_DOCUMENT: &str = "document";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassRow {
    pub target_kind: String,
    pub target_id: Uuid,
    pub class_id: String,
}

#[derive(Debug)]
pub struct NewClass<'a> {
    pub target_kind: &'a str,
    pub target_id: Uuid,
    pub class_id: &'a str,
    pub set_by: &'a str,
    pub reason: Option<&'a str>,
    pub loosened: bool,
}

pub async fn list_classes(
    pool: &PgPool,
    target_kind: &str,
    ids: &[Uuid],
) -> Result<Vec<ClassRow>, sqlx::Error> {
    sqlx::query_as!(
        ClassRow,
        r#"SELECT target_kind, target_id, class_id
             FROM document_classes
            WHERE target_kind = $1 AND target_id = ANY($2)"#,
        target_kind,
        ids,
    )
    .fetch_all(pool)
    .await
}

pub async fn set_class(pool: &PgPool, new: &NewClass<'_>) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO document_classes
                (target_kind, target_id, class_id, set_by, reason, loosened, set_at)
           VALUES ($1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP)
           ON CONFLICT (target_kind, target_id) DO UPDATE
              SET class_id = EXCLUDED.class_id,
                  set_by = EXCLUDED.set_by,
                  reason = EXCLUDED.reason,
                  loosened = EXCLUDED.loosened,
                  set_at = EXCLUDED.set_at"#,
        new.target_kind,
        new.target_id,
        new.class_id,
        new.set_by,
        new.reason,
        new.loosened,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// The rows a class is resolved from: the target's own class, its folder
/// ids root first, and whether an agent created it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lineage {
    pub own: Option<String>,
    pub folder_ids: Vec<Uuid>,
    pub created_by_agent: Option<String>,
}

pub async fn find_document_lineage(
    pool: &PgPool,
    document_id: Uuid,
) -> Result<Option<Lineage>, sqlx::Error> {
    sqlx::query_as!(
        Lineage,
        r#"SELECT
                (SELECT c.class_id FROM document_classes c
                  WHERE c.target_kind = 'document' AND c.target_id = d.id) AS own,
                CASE WHEN d.folder_id IS NULL THEN '{}'::uuid[]
                     ELSE f.ancestor_ids || d.folder_id END AS "folder_ids!",
                d.created_by_agent
             FROM documents d
             LEFT JOIN document_folders f ON f.id = d.folder_id
            WHERE d.id = $1 AND d.deleted_at IS NULL"#,
        document_id,
    )
    .fetch_optional(pool)
    .await
}

pub async fn find_folder_lineage(
    pool: &PgPool,
    folder_id: Uuid,
) -> Result<Option<Lineage>, sqlx::Error> {
    sqlx::query_as!(
        Lineage,
        r#"SELECT
                (SELECT c.class_id FROM document_classes c
                  WHERE c.target_kind = 'folder' AND c.target_id = f.id) AS own,
                f.ancestor_ids AS "folder_ids!",
                NULL::text AS created_by_agent
             FROM document_folders f
            WHERE f.id = $1 AND f.deleted_at IS NULL"#,
        folder_id,
    )
    .fetch_optional(pool)
    .await
}
