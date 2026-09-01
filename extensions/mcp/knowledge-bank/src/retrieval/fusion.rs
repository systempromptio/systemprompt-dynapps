//! Reciprocal rank fusion of the two candidate arms.
//!
//! The vector and text arms return incomparable scores — cosine distance and
//! `ts_rank_cd` — so they are merged by rank, not by value. This is the same
//! fusion the Node "project-context" retrieval service uses, at the same
//! constant, so a ported corpus ranks identically.

use super::candidates::Candidate;
use std::collections::HashMap;

pub(super) const RRF_K: f64 = 60.0;

pub(super) fn fuse(vector_arm: Vec<Candidate>, text_arm: Vec<Candidate>) -> Vec<Candidate> {
    let mut scores: HashMap<String, f64> = HashMap::new();
    let mut by_id: HashMap<String, Candidate> = HashMap::new();

    for arm in [vector_arm, text_arm] {
        for (rank, candidate) in arm.into_iter().enumerate() {
            let contribution = 1.0 / (RRF_K + (rank as f64) + 1.0);
            *scores.entry(candidate.chunk_id.clone()).or_insert(0.0) += contribution;
            by_id.entry(candidate.chunk_id.clone()).or_insert(candidate);
        }
    }

    let mut ranked: Vec<(String, f64)> = scores.into_iter().collect();
    // Why: ties are broken by chunk_id so fusion is deterministic — otherwise
    // an identical query can return a different order run to run, and the
    // REQ-046 precision@5 benchmark stops being reproducible.
    ranked.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });

    ranked
        .into_iter()
        .filter_map(|(chunk_id, _)| by_id.remove(&chunk_id))
        .collect()
}
