//! Joins each `tool_use` to its `tool_result` by `tool_use_id`, so the rail
//! can link the two and the timeline can draw the call as one bar.

use std::collections::HashMap;

/// One event's side of a pairing: its seq, its `tool_use_id` if it has one,
/// and whether it is the use (`true`) or the result (`false`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairKey<'a> {
    pub seq: i64,
    pub tool_use_id: Option<&'a str>,
    pub is_use: bool,
}

pub fn pair_seqs(keys: &[PairKey<'_>]) -> Vec<Option<i64>> {
    let mut uses: HashMap<&str, i64> = HashMap::new();
    let mut results: HashMap<&str, i64> = HashMap::new();
    for key in keys {
        let Some(id) = key.tool_use_id else { continue };
        let side = if key.is_use { &mut uses } else { &mut results };
        side.entry(id).or_insert(key.seq);
    }
    keys.iter()
        .map(|key| {
            let id = key.tool_use_id?;
            let other = if key.is_use { &results } else { &uses };
            other.get(id).copied()
        })
        .collect()
}
