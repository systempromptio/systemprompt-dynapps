//! The `llm` strategy: model-proposed chunk boundaries by line number.
//!
//! The `llm` strategy: ask the model where chunks should start, by line
//! number, and slice the original there. The model never rewrites text — it
//! returns integers — so a bad answer can only produce odd boundaries, which
//! the size discipline then repairs. Any failure returns `None` and the
//! caller falls back to the deterministic transcript splitter.

use super::{Chunk, SplitRequest};
use crate::model::ModelProvider;

pub const LINES_PER_SECTION: usize = 300;

const PROMPT_MEETING_NOTES: &str = "You segment meeting transcripts into retrieval chunks. The \
    input is numbered lines. Return JSON of the form {\"starts\": [1, 12, 34]} listing the line \
    numbers where a new chunk should begin. Start a new chunk only where the discussion moves \
    to a different topic or feature. Keep a question with its answer and a decision with its \
    rationale in the same chunk. Never split inside a coherent exchange. Reply with JSON only.";

const PROMPT_DEFAULT: &str = "You segment a document into retrieval chunks. The input is numbered \
    lines. Return JSON of the form {\"starts\": [1, 12, 34]} listing the line numbers where a new \
    chunk should begin, at natural topic boundaries. Reply with JSON only.";

pub async fn split(model: &dyn ModelProvider, request: SplitRequest<'_>) -> Option<Vec<Chunk>> {
    let lines: Vec<&str> = request.body.lines().collect();
    if lines.is_empty() {
        return None;
    }
    let system = if request.source_type.contains("meeting") {
        PROMPT_MEETING_NOTES
    } else {
        PROMPT_DEFAULT
    };

    let mut starts: Vec<usize> = vec![1];
    for (window_index, window) in lines.chunks(LINES_PER_SECTION).enumerate() {
        let offset = window_index * LINES_PER_SECTION;
        let numbered = window
            .iter()
            .enumerate()
            .map(|(i, line)| format!("{}: {line}", offset + i + 1))
            .collect::<Vec<_>>()
            .join("\n");
        let reply = match model.converse(system, &numbered).await {
            Ok(reply) => reply,
            Err(e) => {
                tracing::warn!(error = %e, "llm chunking failed; falling back to script splitter");
                return None;
            },
        };
        for start in parse_line_numbers(&reply) {
            if start > offset && start <= offset + window.len() {
                starts.push(start);
            }
        }
    }
    starts.sort_unstable();
    starts.dedup();
    Some(split_at_lines(&lines, &starts))
}

#[must_use]
pub fn parse_line_numbers(reply: &str) -> Vec<usize> {
    // JSON: protocol boundary. The model is asked for `{"starts": [...]}`;
    // anything else degrades to scanning the reply for bare integers.
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(extract_json(reply))
        && let Some(starts) = value.get("starts").and_then(serde_json::Value::as_array)
    {
        return starts
            .iter()
            .filter_map(serde_json::Value::as_u64)
            .filter_map(|n| usize::try_from(n).ok())
            .collect();
    }
    reply
        .split(|c: char| !c.is_ascii_digit())
        .filter_map(|token| token.parse::<usize>().ok())
        .collect()
}

fn extract_json(reply: &str) -> &str {
    match (reply.find('{'), reply.rfind('}')) {
        (Some(open), Some(close)) if close > open => &reply[open..=close],
        _ => reply,
    }
}

#[must_use]
pub fn split_at_lines(lines: &[&str], starts: &[usize]) -> Vec<Chunk> {
    let mut boundaries: Vec<usize> = starts
        .iter()
        .map(|s| s.saturating_sub(1))
        .filter(|s| *s < lines.len())
        .collect();
    boundaries.sort_unstable();
    boundaries.dedup();
    if boundaries.first() != Some(&0) {
        boundaries.insert(0, 0);
    }
    boundaries.push(lines.len());
    boundaries
        .windows(2)
        .map(|w| lines[w[0]..w[1]].join("\n"))
        .filter(|text| !text.trim().is_empty())
        .map(|text| Chunk::plain(text.trim()))
        .collect()
}
