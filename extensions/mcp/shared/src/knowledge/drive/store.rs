//! Durable Drive inventory, source requests and processing checkpoints.

pub use super::status::{InventoryEntry, inventory};
use super::types::{DriveFile, FileId, ProjectAssignment};
use crate::knowledge::error::KnowledgeBankError;
use sqlx::PgPool;

#[derive(Debug)]
pub struct SourceState {
    pub drive_id: String,
    pub page_token: Option<String>,
    pub full_inventory: bool,
    pub requested: i64,
    pub completed: i64,
    pub force_requested: bool,
    pub rebuild_requested: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SyncRequest {
    pub force: bool,
    pub rebuild: bool,
}

pub async fn enqueue(
    pool: &PgPool,
    source: &str,
    drive: &str,
    actor: &str,
    request: SyncRequest,
) -> Result<(), KnowledgeBankError> {
    let SyncRequest { force, rebuild } = request;
    let updated = sqlx::query!("INSERT INTO knowledge_drive_sources(source_type,drive_id,requested_by,requested,force_requested,rebuild_requested)
        VALUES($1,$2,$3,1,$4,$5) ON CONFLICT(source_type) DO UPDATE SET
        requested=knowledge_drive_sources.requested+1,requested_by=$3,requested_at=NOW(),
        force_requested=knowledge_drive_sources.force_requested OR $4,
        rebuild_requested=knowledge_drive_sources.rebuild_requested OR $5
        WHERE knowledge_drive_sources.drive_id=EXCLUDED.drive_id", source, drive, actor, force, rebuild).execute(pool).await?.rows_affected();
    if updated == 0 {
        return Err(KnowledgeBankError::Config(
            "A Drive source cannot be repointed; configure a new source ID".to_owned(),
        ));
    }
    Ok(())
}

pub async fn state(pool: &PgPool, source: &str) -> Result<Option<SourceState>, KnowledgeBankError> {
    Ok(sqlx::query_as!(SourceState, "SELECT drive_id,page_token, (last_inventory_at IS NULL OR last_inventory_at < NOW()-INTERVAL '7 days') AS \"full_inventory!\", requested,completed,force_requested,rebuild_requested FROM knowledge_drive_sources WHERE source_type=$1", source).fetch_optional(pool).await?)
}

pub async fn save_file(
    pool: &PgPool,
    source: &str,
    file: &DriveFile,
    generation: Option<&str>,
) -> Result<(), KnowledgeBankError> {
    let version = file.version_key();
    let metadata = serde_json::to_value(file)?;
    sqlx::query!("INSERT INTO knowledge_drive_files(source_type,file_id,metadata,content_version,seen_generation,removed)
        VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT(source_type,file_id) DO UPDATE SET
        metadata=$3,seen_generation=COALESCE($5,knowledge_drive_files.seen_generation),removed=$6,
        status=CASE WHEN knowledge_drive_files.content_version<>$4 OR knowledge_drive_files.removed<>$6 THEN 'pending' ELSE knowledge_drive_files.status END,
        invocation_arn=CASE WHEN knowledge_drive_files.content_version<>$4 THEN NULL ELSE knowledge_drive_files.invocation_arn END,
        extracted_text=CASE WHEN knowledge_drive_files.content_version<>$4 THEN NULL ELSE knowledge_drive_files.extracted_text END,
        retry_at=CASE WHEN knowledge_drive_files.content_version<>$4 THEN NOW() ELSE knowledge_drive_files.retry_at END,
        attempts=CASE WHEN knowledge_drive_files.content_version<>$4 OR knowledge_drive_files.removed<>$6 THEN 0 ELSE knowledge_drive_files.attempts END,
        extraction_attempt=CASE WHEN knowledge_drive_files.content_version<>$4 THEN 0 ELSE knowledge_drive_files.extraction_attempt END,
        content_version=$4",source,file.id,metadata,version,generation,file.trashed).execute(pool).await?;
    Ok(())
}

pub async fn mark_removed(pool: &PgPool, source: &str, id: &str) -> Result<(), KnowledgeBankError> {
    sqlx::query!("UPDATE knowledge_drive_files SET removed=TRUE,status='pending',retry_at=NOW() WHERE source_type=$1 AND file_id=$2",source,id).execute(pool).await?;
    Ok(())
}

pub async fn finish_inventory(
    pool: &PgPool,
    source: &str,
    generation: &str,
    token: &str,
) -> Result<(), KnowledgeBankError> {
    let mut tx = pool.begin().await?;
    sqlx::query!("UPDATE knowledge_drive_files SET removed=TRUE,status='pending',retry_at=NOW() WHERE source_type=$1 AND seen_generation IS DISTINCT FROM $2",source,generation).execute(&mut *tx).await?;
    sqlx::query!("UPDATE knowledge_drive_sources SET page_token=$2,last_inventory_at=NOW() WHERE source_type=$1",source,token).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn cursor(pool: &PgPool, source: &str, token: &str) -> Result<(), KnowledgeBankError> {
    sqlx::query!(
        "UPDATE knowledge_drive_sources SET page_token=$2 WHERE source_type=$1",
        source,
        token
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn complete_listing(
    pool: &PgPool,
    source: &str,
    generation: i64,
    rebuild: bool,
) -> Result<(), KnowledgeBankError> {
    let mut tx = pool.begin().await?;
    if rebuild {
        sqlx::query!("UPDATE knowledge_drive_files SET status='pending',rebuild=TRUE,attempts=0,retry_at=NOW() WHERE source_type=$1 AND NOT removed AND metadata->>'mimeType' NOT IN ('application/vnd.google-apps.folder','application/vnd.google-apps.shortcut')",source).execute(&mut *tx).await?;
    } else {
        sqlx::query!("UPDATE knowledge_drive_files SET status='pending',attempts=0,retry_at=NOW() WHERE source_type=$1 AND status IN ('failed','indexed')",source).execute(&mut *tx).await?;
    }
    sqlx::query!("UPDATE knowledge_drive_sources SET completed=$2,last_sync_at=NOW(),last_error=NULL,
        force_requested=CASE WHEN requested=$2 THEN FALSE ELSE force_requested END,
        rebuild_requested=CASE WHEN requested=$2 THEN FALSE ELSE rebuild_requested END WHERE source_type=$1",source,generation).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn source_error(
    pool: &PgPool,
    source: &str,
    error: &str,
) -> Result<(), KnowledgeBankError> {
    sqlx::query!(
        "UPDATE knowledge_drive_sources SET last_error=$2 WHERE source_type=$1",
        source,
        error
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn files(pool: &PgPool, source: &str) -> Result<Vec<DriveFile>, KnowledgeBankError> {
    let rows = sqlx::query!(
        "SELECT metadata FROM knowledge_drive_files WHERE source_type=$1 AND NOT removed",
        source
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| serde_json::from_value(row.metadata).map_err(Into::into))
        .collect()
}

pub async fn assign(
    pool: &PgPool,
    source: &str,
    id: &str,
    project: &ProjectAssignment,
) -> Result<(), KnowledgeBankError> {
    let mut tx = pool.begin().await?;
    sqlx::query!("UPDATE knowledge_drive_files SET project_id=$3,project_name=$4,folder_path=$5 WHERE source_type=$1 AND file_id=$2",source,id,project.id,project.name,project.path).execute(&mut *tx).await?;
    sqlx::query!("UPDATE knowledge_drive_document_projects SET project_id=$3,project_name=$4,folder_path=$5 WHERE source_type=$1 AND file_id=$2",source,id,project.id,project.name,project.path).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

#[derive(Debug)]
pub struct PendingFile {
    pub file_id: FileId,
    pub metadata: serde_json::Value, // JSON: persisted Google Drive wire document
    pub removed: bool,
    pub rebuild: bool,
    pub extraction_attempt: i32,
    pub extracted_text: Option<String>,
    pub extraction_key: Option<String>,
    pub invocation_arn: Option<String>,
    pub output_uri: Option<String>,
    pub project_id: String,
    pub project_name: String,
    pub folder_path: String,
}

pub async fn pending(pool: &PgPool, source: &str) -> Result<Vec<PendingFile>, KnowledgeBankError> {
    Ok(sqlx::query_as!(PendingFile,"SELECT file_id AS \"file_id: FileId\",metadata,removed,rebuild,extraction_attempt,extracted_text,extraction_key,invocation_arn,output_uri,project_id,project_name,folder_path FROM knowledge_drive_files WHERE source_type=$1 AND status IN ('pending','processing','failed') AND (removed OR attempts<5) AND retry_at<=NOW() ORDER BY removed DESC,retry_at,file_id LIMIT 20",source).fetch_all(pool).await?)
}

pub async fn set_status(
    pool: &PgPool,
    source: &str,
    id: &str,
    status: &str,
    error: Option<&str>,
) -> Result<(), KnowledgeBankError> {
    sqlx::query!("UPDATE knowledge_drive_files SET status=$3,error=$4,
        attempts=CASE WHEN $3='failed' THEN attempts+1 ELSE attempts END,
        retry_at=NOW()+ CASE WHEN $3='failed' THEN INTERVAL '15 minutes' ELSE INTERVAL '30 seconds' END WHERE source_type=$1 AND file_id=$2",source,id,status,error).execute(pool).await?;
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct Checkpoint<'a> {
    pub key: &'a str,
    pub arn: &'a str,
    pub output: &'a str,
    pub local_text: &'a str,
}

pub async fn checkpoint(
    pool: &PgPool,
    source: &str,
    id: &str,
    checkpoint: Checkpoint<'_>,
) -> Result<(), KnowledgeBankError> {
    let Checkpoint {
        key,
        arn,
        output,
        local_text,
    } = checkpoint;
    sqlx::query!("UPDATE knowledge_drive_files SET extraction_key=$3,invocation_arn=$4,output_uri=$5,extracted_text=$6,status='processing',retry_at=NOW()+INTERVAL '30 seconds' WHERE source_type=$1 AND file_id=$2",source,id,key,arn,output,local_text).execute(pool).await?;
    Ok(())
}

pub async fn extracted(
    pool: &PgPool,
    source: &str,
    id: &str,
    key: &str,
    text: &str,
) -> Result<(), KnowledgeBankError> {
    sqlx::query!("UPDATE knowledge_drive_files SET extracted_text=$4,extraction_key=$3,invocation_arn=NULL,output_uri=NULL WHERE source_type=$1 AND file_id=$2",source,id,key,text).execute(pool).await?;
    Ok(())
}

pub async fn indexed(
    pool: &PgPool,
    source: &str,
    file: &PendingFile,
    doc: &str,
) -> Result<(), KnowledgeBankError> {
    let mut tx = pool.begin().await?;
    sqlx::query!("INSERT INTO knowledge_drive_document_projects(doc_id,source_type,file_id,project_id,project_name,folder_path) VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT(doc_id) DO UPDATE SET project_id=$4,project_name=$5,folder_path=$6",doc,source,file.file_id.as_str(),file.project_id,file.project_name,file.folder_path).execute(&mut *tx).await?;
    sqlx::query!("UPDATE knowledge_drive_files SET status='indexed',rebuild=FALSE,error=NULL,attempts=0,indexed_at=NOW() WHERE source_type=$1 AND file_id=$2",source,file.file_id.as_str()).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn delete_document(
    pool: &PgPool,
    source: &str,
    file: &str,
    doc: &str,
) -> Result<(), KnowledgeBankError> {
    let mut tx = pool.begin().await?;
    sqlx::query!(
        "DELETE FROM knowledge_documents WHERE doc_id=$1 AND source_type=$2",
        doc,
        source
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!("UPDATE knowledge_drive_files SET status='removed',extracted_text=NULL,invocation_arn=NULL,output_uri=NULL WHERE source_type=$1 AND file_id=$2",source,file).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn invalidate_extractor(
    pool: &PgPool,
    source: &str,
    version: &str,
) -> Result<(), KnowledgeBankError> {
    sqlx::query!("UPDATE knowledge_drive_files SET extractor_version=$2,extracted_text=NULL,invocation_arn=NULL,output_uri=NULL,status='pending',attempts=0,retry_at=NOW() WHERE source_type=$1 AND extractor_version<>$2 AND NOT removed AND status NOT IN ('folder','shortcut')",source,version).execute(pool).await?;
    Ok(())
}


pub async fn retry_extraction(
    pool: &PgPool,
    source: &str,
    id: &str,
) -> Result<(), KnowledgeBankError> {
    sqlx::query!("UPDATE knowledge_drive_files SET invocation_arn=NULL,output_uri=NULL,extracted_text=NULL,extraction_attempt=extraction_attempt+1 WHERE source_type=$1 AND file_id=$2",source,id).execute(pool).await?;
    Ok(())
}


pub async fn removed_ids(pool: &PgPool, source: &str) -> Result<Vec<String>, KnowledgeBankError> {
    Ok(sqlx::query_scalar!(
        "SELECT file_id FROM knowledge_drive_files WHERE source_type=$1 AND removed",
        source
    )
    .fetch_all(pool)
    .await?)
}


pub async fn request_inventory(pool: &PgPool, source: &str) -> Result<(), KnowledgeBankError> {
    sqlx::query!("UPDATE knowledge_drive_sources SET requested=requested+1,force_requested=TRUE WHERE source_type=$1",source).execute(pool).await?;
    Ok(())
}
