//! Per-document processing: date, chunk, enrich, embed, write.
//!
//! Per-document processing: date → chunk → enrich → embed → write, with
//! every model call routed through the caches first. This is the only place
//! chunk text is assembled, so the invariant that `lines` points into the
//! ORIGINAL text while `text` carries the breadcrumb and enrichment prefix
//! lives here and nowhere else.

use super::chunker::{self, SplitRequest, find_line_range};
use super::dating::{self, DateInputs};
use super::document::SourceDocument;
use super::enrich::{DocumentContext, enrich_chunk};
use super::writer::cache::{find_date, find_embedding, store_date, store_embedding, text_hash};
use super::writer::{ResolvedDate, StoredChunk, upsert_document};
use crate::config::{ChunkStrategy, KnowledgeBankConfig, SourceConfig};
use crate::error::KnowledgeBankError;
use crate::model::ModelProvider;
use crate::retrieval::pgvector_literal;
use sqlx::PgPool;

/// Shared handles for processing one document.
pub struct DocumentJob<'a> {
    pub pool: &'a PgPool,
    pub model: &'a dyn ModelProvider,
    pub config: &'a KnowledgeBankConfig,
    pub source: &'a SourceConfig,
    pub document: &'a SourceDocument,
}

impl std::fmt::Debug for DocumentJob<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocumentJob")
            .field("doc_id", &self.document.doc_id)
            .finish_non_exhaustive()
    }
}

pub async fn process_document(job: &DocumentJob<'_>) -> Result<usize, KnowledgeBankError> {
    let document = job.document;
    let date = resolve_date(job).await?;

    let chunks = chunker::split_document(
        SplitRequest {
            body: &document.body,
            title: &document.title,
            source_type: &document.source_type,
            strategy: job.source.strategy,
            config: job.config.chunking,
        },
        Some(job.model),
    )
    .await;
    if chunks.is_empty() {
        tracing::warn!(doc_id = %document.doc_id, "document produced no chunks; skipped");
        return Ok(0);
    }

    let enrich = job.config.chunking.enrich
        && job.source.strategy == ChunkStrategy::Script
        && chunks.len() > 1;
    let context = enrich.then(|| DocumentContext::build(&document.title, &document.body));

    let mut stored = Vec::with_capacity(chunks.len());
    let mut cursor = 0usize;
    for chunk in &chunks {
        let lines = find_line_range(&document.body, &chunk.text, cursor).map(|(range, next)| {
            cursor = next;
            range
        });
        let mut text = String::new();
        if let Some(context) = &context
            && chunk.text.chars().count() >= job.config.chunking.min_chars
            && let Some(prefix) = enrich_chunk(job.model, context, &chunk.text).await
        {
            text.push_str(&prefix);
            text.push_str("\n\n");
        }
        if !chunk.breadcrumb.is_empty() {
            text.push_str(&chunk.breadcrumb);
            text.push_str("\n\n");
        }
        text.push_str(&chunk.text);

        let embedding = embed_cached(job, &text).await?;
        stored.push(StoredChunk {
            text,
            breadcrumb: chunk.breadcrumb.clone(),
            lines,
            embedding,
        });
    }

    upsert_document(job.pool, document, date, &stored, job.source.archived).await?;
    Ok(stored.len())
}

async fn resolve_date(job: &DocumentJob<'_>) -> Result<ResolvedDate, KnowledgeBankError> {
    let document = job.document;
    if let Some(cached) = find_date(job.pool, &document.doc_id, &document.content_hash).await? {
        return Ok(cached);
    }
    let (date, source) = dating::resolve(
        DateInputs {
            body: &document.body,
            authored: document.authored_date,
            mtime: document.mtime,
            extract_via_llm: job.config.dates.extract_via_llm,
        },
        Some(job.model),
    )
    .await;
    let resolved = ResolvedDate { date, source };
    store_date(job.pool, &document.doc_id, &document.content_hash, resolved).await?;
    Ok(resolved)
}

async fn embed_cached(job: &DocumentJob<'_>, text: &str) -> Result<String, KnowledgeBankError> {
    let hash = text_hash(text);
    let model_id = job.model.embed_model();
    if let Some(literal) = find_embedding(job.pool, model_id, &hash).await? {
        return Ok(literal);
    }
    let vector = job.model.embed(text).await?;
    let literal = pgvector_literal(&vector);
    store_embedding(job.pool, model_id, &hash, &literal).await?;
    Ok(literal)
}
