//! The retrieval backend contract.
//!
//! [`KnowledgeRetriever`] is the seam the server is built against. The
//! production implementation ([`crate::retrieval::PgRetriever`]) is a
//! migration of the Node.js "project-context" retrieval service: hybrid
//! search over pgvector with the configured provider's embeddings,
//! `source_type` pre-filter, candidate over-fetch, reciprocal-rank fusion,
//! and the provider's rerank (BM25 locally, Cohere on Bedrock). There is no
//! stub: an instance that cannot serve does not boot.

use crate::error::KnowledgeBankError;
use crate::principal::Principal;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// Why: ceiling on `top_k`, mirroring the Node retrieval service's cap.
pub const MAX_TOP_K: u8 = 20;

/// Search constraints passed alongside the query.
///
/// `source_types` names the source categories to search; empty means every
/// live source — archive categories are opt-in only and must be listed
/// explicitly. `top_k` caps results at [`MAX_TOP_K`]; `None` takes the
/// backend's configured default.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilter {
    pub source_types: Vec<String>,
    pub top_k: Option<u8>,
}

/// One retrieval result, carrying every field the agent policy (c9) ranks by.
///
/// `uri` points back at the original and `lines` is the `start-end` span
/// inside it; `date` is the document's effective date (`YYYY-MM-DD`) and
/// `date_source` its provenance (`frontmatter` | `llm` | `mtime`). `score`
/// is relevance in `[0.0, 1.0]`, higher is better.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeHit {
    pub doc_id: String,
    pub source_type: String,
    pub source: String,
    pub title: String,
    pub snippet: String,
    pub uri: String,
    pub lines: Option<String>,
    pub date: Option<String>,
    pub date_source: Option<String>,
    pub chunk_index: i32,
    pub chunk_total: i32,
    pub score: f64,
}

/// A knowledge source category the backend can search.
///
/// `id` is the stable value used in [`SearchFilter::source_types`];
/// `doc_count` is the number of documents the *caller* may read, and
/// `last_synced` (RFC 3339) the caller's own last clean sync of the category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSource {
    pub id: String,
    pub name: String,
    pub description: String,
    pub archived: bool,
    pub available: bool,
    pub doc_count: Option<u64>,
    pub last_synced: Option<String>,
}

/// Statistics for the shared index, for administrators.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub documents: u64,
    pub chunks: u64,
    pub grants: u64,
    pub syncing_users: u64,
    pub embedding_cache_rows: u64,
    pub database_bytes: u64,
    pub last_built: Option<String>,
    pub version: Option<String>,
}

/// The backend seam: everything the MCP tools need from retrieval.
///
/// Every method that can surface document content takes the calling
/// [`Principal`]: REQ-052 makes access control a retrieval-time concern, so
/// filtering cannot be bolted on above this trait without a backend that is
/// able to leak. Ingestion is deliberately NOT part of the trait — it feeds
/// the index the retriever reads.
#[async_trait]
pub trait KnowledgeRetriever: Send + Sync {
    async fn search(
        &self,
        principal: &Principal,
        query: &str,
        filter: &SearchFilter,
    ) -> Result<Vec<KnowledgeHit>, KnowledgeBankError>;

    async fn list_sources(
        &self,
        principal: &Principal,
    ) -> Result<Vec<KnowledgeSource>, KnowledgeBankError>;

    async fn index_stats(&self) -> Result<IndexStats, KnowledgeBankError>;
}
