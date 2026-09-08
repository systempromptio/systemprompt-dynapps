//! Publish extracted Drive content through the existing RAG pipeline.

use super::aws::AwsExtraction;
use super::google::GoogleDrive;
use super::types::{DriveFile, FOLDER_MIME, SHORTCUT_MIME};
use super::{extract, store};
use crate::config::SourceKind;
use crate::error::KnowledgeBankError;
use crate::ingest::document::{SourceDocument, content_hash};
use crate::ingest::pipeline::{DocumentJob, process_document};
use crate::ingest::{Ingestor, writer};

pub(super) async fn process(
    ingestor: &Ingestor,
    source: &str,
    pending: &store::PendingFile,
    google: &GoogleDrive,
    aws: &AwsExtraction,
) -> Result<(), KnowledgeBankError> {
    let pool = ingestor.pool();
    let doc_id = SourceDocument::doc_id_for(source, &pending.file_id);
    if pending.removed {
        return store::delete_document(pool, source, &pending.file_id, &doc_id).await;
    }
    let file: DriveFile = serde_json::from_value(pending.metadata.clone())?;
    if file.mime_type == FOLDER_MIME || file.mime_type == SHORTCUT_MIME {
        return Ok(());
    }
    let suffix = super::google::export_format(&file.mime_type).map_or_else(
        || crate::ingest::parsers::suffix_of(&file.name),
        |(_, suffix)| suffix.to_owned(),
    );
    if !extract::supported(&suffix) {
        return store::set_status(
            pool,
            source,
            &file.id,
            "unsupported",
            Some("No extractor for this format; conversion required"),
        )
        .await;
    }
    let Some(mut body) = extract::extract(
        &extract::ExtractionContext {
            pool,
            source,
            google,
            aws,
        },
        &file,
        pending,
    )
    .await?
    else {
        return Ok(());
    };
    let comments = google.comments(&file.id).await?;
    if !comments.is_empty() {
        body.push_str("\n# Drive comments\n");
        body.push_str(&comments);
    }
    if body.trim().is_empty() {
        return Err(KnowledgeBankError::InvalidInput(
            "Extraction returned no readable content".to_owned(),
        ));
    }
    let document = SourceDocument {
        doc_id: doc_id.clone(),
        source_type: source.to_owned(),
        source: "google_drive".to_owned(),
        source_ref: file.id.clone(),
        title: file.name.clone(),
        uri: file.uri(),
        content_hash: content_hash(&body),
        body,
        mtime: file.modified_time,
        authored_date: None,
    };
    publish(ingestor, source, pending, &document).await
}

async fn publish(
    ingestor: &Ingestor,
    source: &str,
    pending: &store::PendingFile,
    document: &SourceDocument,
) -> Result<(), KnowledgeBankError> {
    let pool = ingestor.pool();
    let doc_id = &document.doc_id;
    let config = ingestor
        .config()
        .source(source)
        .filter(|s| s.kind == SourceKind::GoogleDrive)
        .ok_or_else(|| KnowledgeBankError::Config("Drive source category is missing".to_owned()))?;
    let previous = writer::find_content_hash(pool, doc_id).await?;
    if pending.rebuild || previous.as_deref() != Some(document.content_hash.as_str()) {
        let job = DocumentJob {
            pool,
            model: ingestor.model(),
            config: ingestor.config(),
            source: config,
            document,
        };
        if process_document(&job).await? == 0 {
            return Err(KnowledgeBankError::Backend(
                "No chunks were produced".to_owned(),
            ));
        }
    } else {
        sqlx::query!(
            "UPDATE knowledge_documents SET title=$2,uri=$3,mtime=$4 WHERE doc_id=$1",
            doc_id,
            document.title,
            document.uri,
            document.mtime
        )
        .execute(pool)
        .await?;
    }
    store::indexed(pool, source, pending, doc_id).await?;
    writer::replace_configured_grants(pool, doc_id, &config.configured_subjects()).await?;
    writer::stamp_index_state(pool).await?;
    Ok(())
}
