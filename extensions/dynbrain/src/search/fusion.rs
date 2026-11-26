//! Reciprocal Rank Fusion of the two arms, as dynbrain fuses them.
//!
//! A chunk scores `1 / (60 + rank)` in each arm that returned it (rank from
//! 0) and the scores add up. The order is by score, and equal scores keep the
//! order in which the chunks were first seen: the vector arm's, then the text
//! arm's.

use std::collections::HashMap;

use crate::ids::ChunkId;

pub const RRF_K: f64 = 60.0;

pub fn fuse(vector: &[ChunkId], text: &[ChunkId]) -> Vec<(ChunkId, f64)> {
    let mut fused: Vec<(ChunkId, f64)> = Vec::new();
    let mut slots: HashMap<ChunkId, usize> = HashMap::new();
    for arm in [vector, text] {
        for (rank, chunk) in arm.iter().enumerate() {
            let gain = 1.0 / (RRF_K + rank as f64);
            match slots.get(chunk) {
                Some(slot) => fused[*slot].1 += gain,
                None => {
                    slots.insert(*chunk, fused.len());
                    fused.push((*chunk, gain));
                },
            }
        }
    }
    fused.sort_by(|left, right| right.1.total_cmp(&left.1));
    fused
}
