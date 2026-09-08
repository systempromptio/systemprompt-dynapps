//! The production retrieval backend: hybrid search over pgvector, reranked.
//!
//! Ported from the Node "project-context" retrieval service. The pipeline is:
//! embed the query, run a vector arm and a full-text arm each over-fetching
//! `top_k * candidate_multiplier` candidates, fuse them by reciprocal rank,
//! rerank, and truncate to `top_k`.
//!
//! Three things are ours rather than the source's. Access control (REQ-052)
//! runs inside the candidate queries, in the private `candidates` module.
//! The archive opt-in, which the Node server states only in its tool
//! description, is enforced here. And the degradation contract
//! (NFR-DEGRADE-1) is explicit: a failing full-text arm leaves vector-only
//! results, and a failing reranker leaves the fusion order — both warn, neither
//! fails the call.

mod candidates;
mod fusion;
mod sources;

use crate::config::KnowledgeBankConfig;
use crate::error::KnowledgeBankError;
use crate::model::ModelProvider;
use crate::principal::Principal;
use crate::retriever::{
    IndexStats, KnowledgeHit, KnowledgeRetriever, KnowledgeSource, MAX_TOP_K, SearchFilter,
};
use async_trait::async_trait;
use candidates::{Candidate, Subjects};
use sqlx::PgPool;
use std::sync::Arc;

const SNIPPET_CHARS: usize = 480;

#[must_use]
pub fn pgvector_literal(embedding: &[f32]) -> String {
    candidates::to_pgvector_literal(embedding)
}

/// Hybrid retrieval over the Postgres index, with the model provider for
/// query embedding and reranking.
pub struct PgRetriever {
    pool: Arc<PgPool>,
    model: Arc<dyn ModelProvider>,
    config: KnowledgeBankConfig,
}

impl std::fmt::Debug for PgRetriever {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PgRetriever").finish_non_exhaustive()
    }
}

impl PgRetriever {
    pub fn new(
        pool: Arc<PgPool>,
        model: Arc<dyn ModelProvider>,
        config: KnowledgeBankConfig,
    ) -> Self {
        Self {
            pool,
            model,
            config,
        }
    }

    fn resolve_source_types(
        &self,
        requested: &[String],
    ) -> Result<Vec<String>, KnowledgeBankError> {
        if requested.is_empty() {
            return Ok(self.config.live_source_ids());
        }
        for id in requested {
            if !self.config.sources.contains_key(id) {
                return Err(KnowledgeBankError::UnknownSource(id.clone()));
            }
        }
        Ok(requested.to_vec())
    }

    async fn score(
        &self,
        query: &str,
        fused: Vec<Candidate>,
        top_k: usize,
    ) -> Vec<(Candidate, f64)> {
        if self.config.retrieval.rerank_enabled {
            let documents: Vec<String> = fused.iter().map(|c| c.text.clone()).collect();
            match self.model.rerank(query, &documents, top_k).await {
                Ok(ranked) => {
                    return ranked
                        .into_iter()
                        .filter_map(|r| fused.get(r.index).map(|c| (c.clone(), r.relevance_score)))
                        .collect();
                },
                Err(e) => {
                    tracing::warn!(error = %e, "rerank failed; serving the fusion order");
                },
            }
        }
        fused
            .into_iter()
            .take(top_k)
            .enumerate()
            .map(|(rank, c)| (c, 1.0 / (rank as f64 + 1.0)))
            .collect()
    }
}

#[async_trait]
impl KnowledgeRetriever for PgRetriever {
    async fn search(
        &self,
        principal: &Principal,
        query: &str,
        filter: &SearchFilter,
    ) -> Result<Vec<KnowledgeHit>, KnowledgeBankError> {
        let source_types = self.resolve_source_types(&filter.source_types)?;
        if source_types.is_empty() {
            return Ok(Vec::new());
        }
        let top_k = filter
            .top_k
            .unwrap_or(self.config.retrieval.top_k)
            .clamp(1, MAX_TOP_K);
        let over_fetch = i64::from(top_k) * i64::from(self.config.retrieval.candidate_multiplier);
        let subjects = Subjects::from_slice(&principal.subjects());

        let embedding = self.model.embed(query).await?;
        let literal = candidates::to_pgvector_literal(&embedding);

        let scope = candidates::CandidateScope {
            source_types: &source_types,
            subjects: &subjects,
            limit: over_fetch,
            filter,
        };
        let (vector_arm, text_arm) = tokio::join!(
            candidates::vector_candidates(&self.pool, &literal, scope),
            candidates::text_candidates(&self.pool, query, scope),
        );
        let vector_arm = vector_arm?;
        let text_arm = text_arm.unwrap_or_else(|e| {
            tracing::warn!(error = %e, "full-text arm failed; serving vector-only results");
            Vec::new()
        });

        let denied =
            candidates::count_denied_documents(&self.pool, &source_types, &subjects).await?;
        if denied > 0 {
            tracing::info!(
                user_id = %principal.user_id,
                denied_documents = denied,
                "knowledge search filtered documents the caller may not read"
            );
        }

        let fused = fusion::fuse(vector_arm, text_arm);
        if fused.is_empty() {
            return Ok(Vec::new());
        }

        let scored = self.score(query, fused, usize::from(top_k)).await;
        Ok(scored
            .into_iter()
            .take(usize::from(top_k))
            .map(|(candidate, score)| to_hit(candidate, score))
            .collect())
    }

    async fn list_sources(
        &self,
        principal: &Principal,
    ) -> Result<Vec<KnowledgeSource>, KnowledgeBankError> {
        sources::list_visible_sources(&self.pool, &self.config, principal).await
    }

    async fn index_stats(&self) -> Result<IndexStats, KnowledgeBankError> {
        sources::read_index_stats(&self.pool).await
    }
}

fn to_hit(candidate: Candidate, score: f64) -> KnowledgeHit {
    KnowledgeHit {
        doc_id: candidate.doc_id,
        source_type: candidate.source_type,
        source: candidate.source,
        title: candidate.title,
        snippet: snippet(&candidate.text),
        uri: candidate.uri,
        lines: candidate.lines,
        date: candidate.doc_date.map(|d| d.format("%Y-%m-%d").to_string()),
        date_source: candidate.date_source,
        chunk_index: candidate.chunk_index,
        chunk_total: candidate.chunk_total,
        score: score.clamp(0.0, 1.0),
    }
}

fn snippet(text: &str) -> String {
    if text.chars().count() <= SNIPPET_CHARS {
        return text.to_owned();
    }
    let truncated: String = text.chars().take(SNIPPET_CHARS).collect();
    format!("{truncated}…")
}
