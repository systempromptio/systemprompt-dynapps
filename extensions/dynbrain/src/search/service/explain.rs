//! The search broken into its stages, for the admin's search laboratory.
//!
//! [`Retriever::stage`] is what [`Retriever::retrieve`] runs for the hybrid
//! search, so the order the laboratory shows is the order the operations
//! return. [`Retriever::explain`] adds, for each candidate, its rank and
//! score in each arm and what the deduplication did with it.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::{Passage, Retrieval, RetrievalMode, RetrievalRequest, Retriever};
use crate::error::BrainError;
use crate::ids::ChunkId;
use crate::repository::admin::ArmRow;
use crate::search::dedup::{Dropped, dedup_explained};
use crate::search::fusion::fuse;
use crate::search::query::{QueryVector, cache_key};
use crate::search::rerank::{ScoreSource, ScoringTrace, rerank};

#[derive(Debug)]
pub(super) struct Staged {
    pub(super) near: Vec<ChunkId>,
    pub(super) words: Vec<ChunkId>,
    pub(super) fused: Vec<(ChunkId, f64)>,
    pub(super) ranked: Vec<Passage>,
    pub(super) trace: ScoringTrace,
    pub(super) arms: Duration,
    pub(super) reranking: Duration,
}

/// The rank (from 1) and the score a chunk had in one arm.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArmScore {
    pub rank: usize,
    pub value: f64,
}

/// What became of a candidate: kept, or dropped for a reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Kept,
    SameDocument,
    NearDuplicate,
    OverLimit,
}

/// One candidate with the scores that put it where it is.
#[derive(Debug, Clone, PartialEq)]
pub struct ExplainRow {
    pub passage: Passage,
    pub vector: Option<ArmScore>,
    pub text: Option<ArmScore>,
    pub fused: f64,
    pub rerank: Option<f64>,
    pub outcome: Outcome,
}

/// Milliseconds spent on each step of one search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timings {
    pub embedding: u64,
    pub arms: u64,
    pub rerank: u64,
}

/// A search with its candidates, in ranked order, and what it returns.
#[derive(Debug, Clone, PartialEq)]
pub struct Explained {
    pub mode: RetrievalMode,
    pub trace: ScoringTrace,
    pub rows: Vec<ExplainRow>,
    pub kept: Vec<Passage>,
    pub cached: bool,
    pub timings: Timings,
}

impl Retriever {
    pub(super) async fn stage(
        &self,
        request: &RetrievalRequest<'_>,
        fingerprint: &str,
        vector: &QueryVector,
        rerank_wanted: bool,
    ) -> Result<Staged, BrainError> {
        let started = Instant::now();
        let pool = self.settings.candidate_pool;
        let near = if self.repo.get_foreign_vectors(fingerprint).await? {
            tracing::error!(
                fingerprint,
                "vector search is off: chunks are embedded with another configuration"
            );
            Vec::new()
        } else {
            let effort = self.settings.ef_search.max(pool);
            self.repo
                .list_vector_matches(request.scope, vector, pool, effort)
                .await?
        };
        let words = self
            .repo
            .list_text_matches(request.scope, request.query, pool)
            .await?;
        let arms = started.elapsed();
        let fused = fuse(&near, &words);
        let ids: Vec<ChunkId> = fused.iter().map(|(chunk, _)| *chunk).collect();
        let mut rows: HashMap<ChunkId, Passage> = self
            .repo
            .list_passages(&ids)
            .await?
            .into_iter()
            .filter_map(|passage| passage.chunk.map(|chunk| (chunk, passage)))
            .collect();
        let passages: Vec<Passage> = fused
            .iter()
            .filter_map(|(chunk, score)| {
                rows.remove(chunk).map(|passage| Passage {
                    score: Some(*score),
                    ..passage
                })
            })
            .collect();
        let rank_started = Instant::now();
        let reranker = self.reranker.as_deref().filter(|_| rerank_wanted);
        let (ranked, trace) = rerank(reranker, &self.breaker, request.query, passages).await;
        Ok(Staged {
            near,
            words,
            fused,
            ranked,
            trace,
            arms,
            reranking: rank_started.elapsed(),
        })
    }

    pub async fn explain(
        &self,
        request: &RetrievalRequest<'_>,
        rerank_wanted: bool,
    ) -> Result<Explained, BrainError> {
        let started = Instant::now();
        let embedder = self.embedder.get().filter(|_| !request.query.is_empty());
        let (cached, staged_vector) = match embedder {
            Some(embedder) => {
                let fingerprint = embedder.fingerprint();
                let cached = self
                    .repo
                    .find_cached_query(&cache_key(&fingerprint, request.query).hash)
                    .await?
                    .is_some();
                let vector = self
                    .query_vector(embedder.as_ref(), request.query)
                    .await?
                    .map(|vector| (fingerprint, vector));
                (cached, vector)
            },
            None => (false, None),
        };
        let embedding = started.elapsed();
        let Some((fingerprint, vector)) = staged_vector else {
            let found = self.keywords(request).await?;
            return Ok(by_keywords(found, cached, embedding));
        };
        let staged = self
            .stage(request, &fingerprint, &vector, rerank_wanted)
            .await?;
        let ids: Vec<i64> = staged.fused.iter().map(|(chunk, _)| chunk.0).collect();
        let scores = self
            .repo
            .list_arm_scores(&ids, &vector, request.query)
            .await?;
        let deduped = dedup_explained(staged.ranked.clone(), request.prefer_lang);
        let mut kept = deduped.kept;
        let over: Vec<Option<ChunkId>> = kept
            .split_off(request.limit.min(kept.len()))
            .iter()
            .map(|passage| passage.chunk)
            .collect();
        Ok(Explained {
            mode: RetrievalMode::Hybrid,
            rows: rows_of(&staged, &scores, &deduped.dropped, &over),
            trace: staged.trace,
            kept,
            cached,
            timings: timings(embedding, staged.arms, staged.reranking),
        })
    }
}

fn by_keywords(found: Retrieval, cached: bool, embedding: Duration) -> Explained {
    let rows = found
        .passages
        .iter()
        .map(|passage| ExplainRow {
            passage: passage.clone(),
            vector: None,
            text: None,
            fused: 0.0,
            rerank: None,
            outcome: Outcome::Kept,
        })
        .collect();
    Explained {
        mode: RetrievalMode::Keyword,
        trace: found.trace,
        rows,
        kept: found.passages,
        cached,
        timings: timings(embedding, Duration::ZERO, Duration::ZERO),
    }
}

fn rows_of(
    staged: &Staged,
    scores: &[ArmRow],
    dropped: &[(Passage, Dropped)],
    over: &[Option<ChunkId>],
) -> Vec<ExplainRow> {
    let fused: HashMap<ChunkId, f64> = staged.fused.iter().copied().collect();
    let cross = staged.trace.scoring == ScoreSource::CrossEncoder;
    staged
        .ranked
        .iter()
        .map(|passage| {
            let chunk = passage.chunk;
            let arm = scores.iter().find(|row| Some(ChunkId(row.id)) == chunk);
            let why = dropped
                .iter()
                .find(|(lost, _)| lost.chunk == chunk)
                .map(|(_, why)| *why);
            ExplainRow {
                passage: passage.clone(),
                vector: chunk
                    .and_then(|id| staged.near.iter().position(|near| *near == id))
                    .map(|at| ArmScore {
                        rank: at + 1,
                        value: arm.and_then(|row| row.cosine).unwrap_or_default(),
                    }),
                text: chunk
                    .and_then(|id| staged.words.iter().position(|word| *word == id))
                    .map(|at| ArmScore {
                        rank: at + 1,
                        value: arm.map_or(0.0, |row| row.ts_rank),
                    }),
                fused: chunk
                    .and_then(|id| fused.get(&id).copied())
                    .unwrap_or_default(),
                rerank: passage.score.filter(|_| cross),
                outcome: match (why, over.contains(&chunk)) {
                    (Some(Dropped::SameDocument), _) => Outcome::SameDocument,
                    (Some(Dropped::NearDuplicate), _) => Outcome::NearDuplicate,
                    (None, true) => Outcome::OverLimit,
                    (None, false) => Outcome::Kept,
                },
            }
        })
        .collect()
}

fn timings(embedding: Duration, arms: Duration, reranking: Duration) -> Timings {
    Timings {
        embedding: millis(embedding),
        arms: millis(arms),
        rerank: millis(reranking),
    }
}

fn millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}
