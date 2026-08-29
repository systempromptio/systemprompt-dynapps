//! Offline ingestion: read a source system, chunk, embed, and write the index.
//!
//! Ingestion is deliberately not part of
//! [`crate::retriever::KnowledgeRetriever`] — it is an offline concern that
//! feeds the index the retriever reads, and nothing on the serving path should
//! be able to trigger it.
//!
//! REQ-048's "respect source-system permissions" is satisfied here rather
//! than at query time: each connector reads its source object's own
//! restrictions and [`writer::upsert_document`] writes them as grants. That
//! makes the propagation lag exactly the sync interval, which is the honest
//! answer to REQ-052's "permission changes propagate".

pub mod atlassian;
pub mod chunker;
pub mod writer;

use crate::bedrock::{BedrockClient, embed_text};
use crate::config::KnowledgeBankConfig;
use crate::error::KnowledgeBankError;
use sqlx::PgPool;
use writer::SourceDocument;

/// What one ingestion pass did, for the job log.
#[derive(Debug, Clone, Copy, Default)]
pub struct IngestReport {
    pub documents_seen: usize,
    pub documents_written: usize,
    pub documents_skipped: usize,
    pub chunks_written: usize,
}

pub async fn ingest_documents(
    pool: &PgPool,
    bedrock: &BedrockClient,
    config: &KnowledgeBankConfig,
    documents: &[SourceDocument],
) -> Result<IngestReport, KnowledgeBankError> {
    let mut report = IngestReport {
        documents_seen: documents.len(),
        ..IngestReport::default()
    };

    for document in documents {
        if writer::is_unchanged(pool, &document.doc_id, &document.content_hash).await? {
            report.documents_skipped += 1;
            continue;
        }
        if document.grants.is_empty() {
            tracing::warn!(
                doc_id = %document.doc_id,
                "indexing with no grants: source permissions could not be determined, \
                 so this document will be reachable by nobody"
            );
        }

        let chunks = chunker::split_markdown(&document.body, config.chunking);
        let mut texts = Vec::with_capacity(chunks.len());
        let mut embeddings = Vec::with_capacity(chunks.len());
        for chunk in &chunks {
            let text = if chunk.breadcrumb.is_empty() {
                chunk.text.clone()
            } else {
                format!("{}\n\n{}", chunk.breadcrumb, chunk.text)
            };
            let vector = embed_text(bedrock, &text).await?;
            embeddings.push(crate::retrieval::pgvector_literal(&vector));
            texts.push((text, chunk.breadcrumb.clone()));
        }

        writer::upsert_document(pool, document, &texts, &embeddings).await?;
        report.documents_written += 1;
        report.chunks_written += chunks.len();
    }

    Ok(report)
}
