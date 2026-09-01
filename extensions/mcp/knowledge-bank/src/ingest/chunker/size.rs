//! Size discipline applied after every strategy.
//!
//! Size discipline applied after every strategy: merge tiny fragments, then
//! enforce the hard ceiling. Tiny fragments hurt retrieval (no context);
//! oversized ones exceed the embedding model's input and dilute relevance.

use super::Chunk;
use crate::config::ChunkingConfig;

#[must_use]
pub fn discipline(chunks: Vec<Chunk>, config: ChunkingConfig) -> Vec<Chunk> {
    let merged = merge_small(chunks, config);
    merged
        .into_iter()
        .flat_map(|chunk| enforce_ceiling(&chunk, config))
        .filter(|chunk| !chunk.text.trim().is_empty())
        .collect()
}

#[must_use]
pub fn merge_small(chunks: Vec<Chunk>, config: ChunkingConfig) -> Vec<Chunk> {
    let mut out: Vec<Chunk> = Vec::new();
    for chunk in chunks {
        let too_small = chunk.text.chars().count() < config.min_chars;
        match out.last_mut() {
            Some(previous)
                if too_small
                    && previous.text.chars().count() + chunk.text.chars().count()
                        <= config.max_chars =>
            {
                previous.text.push_str("\n\n");
                previous.text.push_str(&chunk.text);
            },
            _ => out.push(chunk),
        }
    }
    out
}

#[must_use]
pub fn enforce_ceiling(chunk: &Chunk, config: ChunkingConfig) -> Vec<Chunk> {
    let ceiling = config.ceiling();
    if chunk.text.chars().count() <= ceiling {
        return vec![chunk.clone()];
    }
    let mut out = Vec::new();
    let mut rest = chunk.text.as_str();
    while rest.chars().count() > ceiling {
        let cut = break_point(rest, config.max_chars);
        let (head, tail) = rest.split_at(cut);
        out.push(Chunk {
            text: head.trim().to_owned(),
            breadcrumb: chunk.breadcrumb.clone(),
        });
        rest = tail.trim_start();
    }
    if !rest.trim().is_empty() {
        out.push(Chunk {
            text: rest.trim().to_owned(),
            breadcrumb: chunk.breadcrumb.clone(),
        });
    }
    out
}

#[must_use]
pub fn split_by_sentences(text: &str, target: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for sentence in sentences(text) {
        if !current.is_empty() && current.chars().count() + sentence.chars().count() > target {
            out.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(sentence);
    }
    if !current.trim().is_empty() {
        out.push(current);
    }
    out
}

fn sentences(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let bytes = text.as_bytes();
    for (i, c) in text.char_indices() {
        if matches!(c, '.' | '!' | '?') {
            let next = i + c.len_utf8();
            if next >= bytes.len() || bytes[next].is_ascii_whitespace() {
                let piece = text[start..next].trim();
                if !piece.is_empty() {
                    out.push(piece);
                }
                start = next;
            }
        }
    }
    let tail = text[start..].trim();
    if !tail.is_empty() {
        out.push(tail);
    }
    out
}

// Why: byte index, not char index — the caller splits a &str with it. Every
// candidate below is taken from a char-boundary API, so the returned index
// never lands inside a multi-byte character.
fn break_point(text: &str, target: usize) -> usize {
    let limit = text
        .char_indices()
        .nth(target)
        .map_or_else(|| text.len(), |(i, _)| i);
    let window = &text[..limit];

    for terminator in ['.', '!', '?'] {
        if let Some(found) = window.rfind(terminator) {
            let after = found + terminator.len_utf8();
            if after > limit / 2 {
                return after;
            }
        }
    }
    window
        .rfind(char::is_whitespace)
        .filter(|found| *found > limit / 2)
        .map_or(limit, |found| found + 1)
}
