//! The text an answer is written from, assembled as `_build_context` does.
//!
//! Every source document gets the same share, 25,000 characters by default.
//! A document that fits is given whole. One that does not is given its own
//! chunks most relevant to the question, each one whole, back in reading
//! order, with `[...]` wherever a chunk was left out. Only the best chunk is
//! ever cut, when it alone overflows the share, and the cut is marked; a
//! document with no embedded chunks is cut to its head and marked, which is
//! logged because it is the degraded path. Blocks are numbered by the
//! position of their source, `[n] title (path, lang=xx)`, `lang=?` when the
//! language is unknown.

use std::collections::HashMap;

use crate::ids::ChunkId;

pub const QUOTA_CHARS: usize = 25_000;
pub const GAP_MARK: &str = "[...]";
pub const TRUNCATED_MARK: &str = "\n[...truncated...]";
pub const BLOCK_SEPARATOR: &str = "\n\n---\n\n";

/// A chunk of a source document, in the order of its relevance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextChunk {
    pub id: ChunkId,
    pub sequence: i32,
    pub content: String,
}

/// One source of an answer and the text assembled for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextSource {
    pub title: String,
    pub path: String,
    pub lang: String,
    pub text: String,
}

pub fn document_text(markdown: &str, budget: usize, ranked: &[ContextChunk]) -> String {
    if !markdown.is_empty() && chars(markdown) <= budget {
        return markdown.to_owned();
    }
    let selected = select_chunks(ranked, budget);
    if !selected.is_empty() || markdown.is_empty() {
        return selected;
    }
    tracing::warn!(
        chars = chars(markdown),
        "answer context: a document with no embedded chunks is cut to its head"
    );
    format!("{}{TRUNCATED_MARK}", head(markdown, budget))
}

pub fn select_chunks(ranked: &[ContextChunk], budget: usize) -> String {
    let mut reading: Vec<&ContextChunk> = ranked.iter().collect();
    reading.sort_by_key(|chunk| (chunk.sequence, chunk.id));
    let position: HashMap<ChunkId, usize> = reading
        .iter()
        .enumerate()
        .map(|(at, chunk)| (chunk.id, at))
        .collect();
    let mut picked: Vec<(usize, String)> = Vec::new();
    let mut used = 0;
    for chunk in ranked {
        let Some(at) = position.get(&chunk.id).copied() else {
            continue;
        };
        let size = chars(&chunk.content);
        if picked.is_empty() {
            let text = if size > budget {
                format!("{}{TRUNCATED_MARK}", head(&chunk.content, budget))
            } else {
                chunk.content.clone()
            };
            used = chars(&text);
            picked.push((at, text));
            continue;
        }
        if used + size + 1 > budget {
            continue;
        }
        picked.push((at, chunk.content.clone()));
        used += size + 1;
    }
    picked.sort_by_key(|(at, _)| *at);
    let mut parts: Vec<String> = Vec::new();
    let mut previous: Option<usize> = None;
    for (at, text) in picked {
        if previous.is_some_and(|before| at != before + 1) {
            parts.push(GAP_MARK.to_owned());
        }
        parts.push(text);
        previous = Some(at);
    }
    parts.join("\n")
}

pub fn blocks(sources: &[ContextSource]) -> String {
    sources
        .iter()
        .enumerate()
        .filter(|(_, source)| !source.text.is_empty())
        .map(|(index, source)| {
            let lang = if source.lang.is_empty() {
                "?"
            } else {
                source.lang.as_str()
            };
            format!(
                "[{}] {} ({}, lang={lang})\n{}",
                index + 1,
                source.title,
                source.path,
                source.text
            )
        })
        .collect::<Vec<_>>()
        .join(BLOCK_SEPARATOR)
}

fn chars(text: &str) -> usize {
    text.chars().count()
}

fn head(text: &str, budget: usize) -> String {
    text.chars().take(budget).collect()
}
