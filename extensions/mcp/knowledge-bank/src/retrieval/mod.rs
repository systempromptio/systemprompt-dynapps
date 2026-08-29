//! The production retrieval backend: hybrid search over pgvector, reranked.
//!
//! Ported from the Node "project-context" retrieval service. The pipeline is:
//! embed the query with Titan v2, run a vector arm and a full-text arm each
//! over-fetching `top_k * candidate_multiplier` candidates, fuse them by
//! reciprocal rank, rerank with Cohere, and truncate to `top_k`.
//!
//! Two things are ours rather than the source's. Access control (REQ-052)
//! runs inside the candidate queries, in the private `candidates` module. And
//! the archive
//! opt-in, which the Node server states only in its tool description, is
//! enforced here: naming no source type searches the live ones, and an
//! archived category is searched only when it is asked for by name.

mod candidates;
mod fusion;
mod sources;

use crate::bedrock::{BedrockClient, embed_text, rerank};
use crate::config::KnowledgeBankConfig;
use crate::error::KnowledgeBankError;
use crate::principal::Principal;
use crate::retriever::{
    IndexStats, KnowledgeHit, KnowledgeRetriever, KnowledgeSource, MAX_TOP_K, SearchFilter,
};
use async_trait::async_trait;
use candidates::Subjects;
use sqlx::PgPool;
use std::sync::Arc;

const SNIPPET_CHARS: usize = 480;

pub fn pgvector_literal(embedding: &[f32]) -> String {
    candidates::to_pgvector_literal(embedding)
}

/// Hybrid retrieval over the Postgres index, with Bedrock for embed + rerank.
#[derive(Debug)]
pub struct PgRetriever {
    pool: Arc<PgPool>,
    bedrock: Arc<BedrockClient>,
    config: KnowledgeBankConfig,
}

impl PgRetriever {
    pub const fn new(
        pool: Arc<PgPool>,
        bedrock: Arc<BedrockClient>,
        config: KnowledgeBankConfig,
    ) -> Self {
        Self {
            pool,
            bedrock,
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

        let embedding = embed_text(&self.bedrock, query).await?;
        let literal = candidates::to_pgvector_literal(&embedding);

        let (vector_arm, text_arm) = tokio::try_join!(
            candidates::vector_candidates(
                &self.pool,
                &literal,
                &source_types,
                &subjects,
                over_fetch
            ),
            candidates::text_candidates(&self.pool, query, &source_types, &subjects, over_fetch),
        )?;

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

        let scored = if self.config.retrieval.rerank_enabled {
            let documents: Vec<String> = fused.iter().map(|c| c.text.clone()).collect();
            let ranked = rerank(&self.bedrock, query, &documents, usize::from(top_k)).await?;
            ranked
                .into_iter()
                .filter_map(|r| fused.get(r.index).map(|c| (c.clone(), r.relevance_score)))
                .collect::<Vec<_>>()
        } else {
            fused
                .iter()
                .take(usize::from(top_k))
                .enumerate()
                .map(|(rank, c)| (c.clone(), 1.0 / (rank as f64 + 1.0)))
                .collect()
        };

        Ok(scored
            .into_iter()
            .take(usize::from(top_k))
            .map(|(candidate, score)| KnowledgeHit {
                source_type: candidate.source_type,
                source: candidate.source,
                title: candidate.title,
                snippet: snippet(&candidate.text),
                uri: candidate.lines.as_ref().map_or_else(
                    || candidate.uri.clone(),
                    |l| format!("{}#{l}", candidate.uri),
                ),
                score: score.clamp(0.0, 1.0),
            })
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

fn snippet(text: &str) -> String {
    if text.chars().count() <= SNIPPET_CHARS {
        return text.to_owned();
    }
    let truncated: String = text.chars().take(SNIPPET_CHARS).collect();
    format!("{truncated}…")
}
