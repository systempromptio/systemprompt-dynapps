//! Shared-source processing status and paginated inventory for MCP clients.

use crate::error::KnowledgeBankError;
use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Serialize)]
pub struct DriveStatus {
    pub source_type: String,
    pub requested: i64,
    pub completed: i64,
    pub last_sync_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_error: Option<String>,
    pub status: String,
    pub project_id: String,
    pub project_name: String,
    pub files: i64,
}

pub async fn list(pool: &PgPool) -> Result<Vec<DriveStatus>, KnowledgeBankError> {
    Ok(sqlx::query_as!(DriveStatus,"SELECT s.source_type,s.requested,s.completed,s.last_sync_at,s.last_error,
        COALESCE(f.status,'empty') AS \"status!\",COALESCE(f.project_id,'unassigned') AS \"project_id!\",
        COALESCE(f.project_name,'Unassigned') AS \"project_name!\",COUNT(f.file_id) AS \"files!\"
        FROM knowledge_drive_sources s LEFT JOIN knowledge_drive_files f ON f.source_type=s.source_type
        GROUP BY s.source_type,f.status,f.project_id,f.project_name ORDER BY s.source_type,f.project_name,f.status").fetch_all(pool).await?)
}

#[derive(Debug, Serialize)]
pub struct InventoryEntry {
    pub source_type: String,
    pub file_id: String,
    pub project_id: String,
    pub project_name: String,
    pub folder_path: String,
    pub status: String,
    pub file_name: String,
    pub mime_type: String,
    pub uri: String,
    pub error: Option<String>,
}

pub async fn inventory(
    pool: &PgPool,
    source: &str,
    after: &str,
    limit: i64,
) -> Result<Vec<InventoryEntry>, KnowledgeBankError> {
    Ok(sqlx::query_as!(InventoryEntry,"SELECT source_type,file_id,project_id,project_name,folder_path,status,
        COALESCE(metadata->>'name','') AS \"file_name!\",COALESCE(metadata->>'mimeType','') AS \"mime_type!\",
        COALESCE(metadata->>'webViewLink','https://drive.google.com/file/d/' || file_id || '/view') AS \"uri!\",error FROM knowledge_drive_files WHERE source_type=$1 AND file_id>$2 ORDER BY file_id LIMIT $3",source,after,limit).fetch_all(pool).await?)
}
