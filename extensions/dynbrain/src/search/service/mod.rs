//! [`Retriever`]: one search, the way dynbrain's `_hybrid_search` runs it.
//!
//! It is the service the operations call, and the one the admin's search
//! laboratory extends: [`Retrieval`] carries the passages, the mode, the
//! [`ScoringTrace`] and the query vector, which the answer reuses to pick the
//! chunks of each document without a second embedding call.

use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use systemprompt::identifiers::UserId;

use super::confidence::WEAK_MATCH_THRESHOLD;
use super::context::QUOTA_CHARS;
use super::dedup::dedup;
use super::filter::PropertyFilter;
use super::query::{EmbedderSlot, QueryVector, cache_key, literal};
use super::rerank::{FusionReason, RerankBreaker, Reranker, ScoringTrace};
use crate::error::BrainError;
use crate::ids::{ArticleId, ChunkId, SpaceId};
use crate::ingest::provider::{DIMENSIONS, Embedder};
use crate::repository::BrainRepository;

/// How a search is tuned. The defaults are dynbrain's.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RetrievalSettings {
    pub candidate_pool: i64,
    pub ef_search: i64,
    pub weak_match_threshold: f64,
    pub context_quota_chars: usize,
    pub query_timeout: Duration,
    pub query_retries: u32,
    pub cache_days: i32,
}

impl Default for RetrievalSettings {
    fn default() -> Self {
        Self {
            candidate_pool: 100,
            ef_search: 200,
            weak_match_threshold: WEAK_MATCH_THRESHOLD,
            context_quota_chars: QUOTA_CHARS,
            query_timeout: Duration::from_secs(6),
            query_retries: 1,
            cache_days: 30,
        }
    }
}

/// Who searches and what the search may cover. `lang` keeps the search to
/// the spaces written in that language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchScope {
    pub reader: UserId,
    pub space: Option<SpaceId>,
    pub lang: Option<String>,
    pub filter: PropertyFilter,
}

/// A result: a chunk of a variant (or, by keywords, the variant itself), with
/// the score that ranked it.
#[derive(Debug, Clone, PartialEq)]
pub struct Passage {
    pub chunk: Option<ChunkId>,
    pub article: ArticleId,
    pub space: SpaceId,
    pub lang: String,
    pub title: String,
    pub description: String,
    pub content: String,
    pub score: Option<f64>,
}

/// One search to run.
#[derive(Debug, Clone, Copy)]
pub struct RetrievalRequest<'a> {
    pub query: &'a str,
    pub scope: &'a SearchScope,
    pub limit: usize,
    pub prefer_lang: Option<&'a str>,
}

/// How the results were found: both arms fused, or keywords alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RetrievalMode {
    Hybrid,
    Keyword,
}

/// What a search found and how it scored it.
#[derive(Debug, Clone, PartialEq)]
pub struct Retrieval {
    pub mode: RetrievalMode,
    pub passages: Vec<Passage>,
    pub trace: ScoringTrace,
    pub vector: Option<QueryVector>,
}

mod explain;

pub use explain::{ArmScore, ExplainRow, Explained, Outcome, Timings};

/// The hybrid search over the native index.
#[derive(Debug, Clone)]
pub struct Retriever {
    repo: BrainRepository,
    embedder: EmbedderSlot,
    reranker: Option<Arc<dyn Reranker>>,
    breaker: Arc<RerankBreaker>,
    settings: RetrievalSettings,
}

impl Retriever {
    pub fn new(repo: BrainRepository) -> Self {
        Self {
            repo,
            embedder: EmbedderSlot::default(),
            reranker: None,
            breaker: Arc::default(),
            settings: RetrievalSettings::default(),
        }
    }

    pub fn with_embedder(self, embedder: EmbedderSlot) -> Self {
        Self { embedder, ..self }
    }

    pub fn with_reranker(self, reranker: Option<Arc<dyn Reranker>>) -> Self {
        Self { reranker, ..self }
    }

    pub fn with_settings(self, settings: RetrievalSettings) -> Self {
        Self { settings, ..self }
    }

    pub const fn settings(&self) -> RetrievalSettings {
        self.settings
    }

    pub async fn retrieve(&self, request: &RetrievalRequest<'_>) -> Result<Retrieval, BrainError> {
        let embedder = self.embedder.get().filter(|_| !request.query.is_empty());
        let Some(embedder) = embedder else {
            return self.keywords(request).await;
        };
        match self.query_vector(embedder.as_ref(), request.query).await? {
            Some(vector) => self.hybrid(request, &embedder.fingerprint(), vector).await,
            None => self.keywords(request).await,
        }
    }

    async fn hybrid(
        &self,
        request: &RetrievalRequest<'_>,
        fingerprint: &str,
        vector: QueryVector,
    ) -> Result<Retrieval, BrainError> {
        let staged = self.stage(request, fingerprint, &vector, true).await?;
        let mut kept = dedup(staged.ranked, request.prefer_lang);
        kept.truncate(request.limit);
        Ok(Retrieval {
            mode: RetrievalMode::Hybrid,
            passages: kept,
            trace: staged.trace,
            vector: Some(vector),
        })
    }

    async fn keywords(&self, request: &RetrievalRequest<'_>) -> Result<Retrieval, BrainError> {
        let pattern = format!("%{}%", escape_like(request.query));
        let passages = self
            .repo
            .list_keyword_matches(request.scope, &pattern, request.limit as i64)
            .await?;
        Ok(Retrieval {
            mode: RetrievalMode::Keyword,
            passages,
            trace: ScoringTrace::fused(FusionReason::NotRequested),
            vector: None,
        })
    }

    async fn query_vector(
        &self,
        embedder: &dyn Embedder,
        query: &str,
    ) -> Result<Option<QueryVector>, BrainError> {
        let key = cache_key(&embedder.fingerprint(), query);
        if let Some(cached) = self.repo.find_cached_query(&key.hash).await? {
            return Ok(Some(QueryVector(cached)));
        }
        let Some(vector) = self.embed_query(embedder, query).await else {
            return Ok(None);
        };
        self.repo.insert_cached_query(&key, query, &vector).await?;
        self.repo
            .delete_stale_queries(self.settings.cache_days)
            .await?;
        Ok(Some(vector))
    }

    async fn embed_query(&self, embedder: &dyn Embedder, query: &str) -> Option<QueryVector> {
        let texts = [query.to_owned()];
        for _ in 0..=self.settings.query_retries {
            match tokio::time::timeout(self.settings.query_timeout, embedder.embed(&texts)).await {
                Ok(Ok(done)) => {
                    return done
                        .vectors
                        .first()
                        .map(Vec::as_slice)
                        .filter(|vector| vector.len() == DIMENSIONS)
                        .map(literal);
                },
                Ok(Err(failure)) if !failure.is_transient() => {
                    tracing::warn!(error = %failure, "the query could not be embedded; searching by keywords");
                    return None;
                },
                Ok(Err(failure)) => {
                    tracing::warn!(error = %failure, "embedding the query failed");
                },
                Err(elapsed) => {
                    tracing::warn!(error = %elapsed, "embedding the query timed out");
                },
            }
        }
        None
    }
}

fn escape_like(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}
