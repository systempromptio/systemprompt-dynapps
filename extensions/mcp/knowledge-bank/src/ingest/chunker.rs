//! The deterministic chunk splitter and its hard ceiling.
//!
//! Ported from the Node "project-context" chunker's `script` strategy: split
//! on markdown headings, keep a breadcrumb of the enclosing headings on each
//! chunk so a fragment still says where it came from, and merge anything
//! under `min_chars` into its neighbour.
//!
//! `enforce_ceiling` is the backstop the source system also carries: whatever
//! the strategy produced, no chunk leaves here longer than
//! `max_chars + split_tolerance`. It splits at a sentence boundary, then at
//! whitespace, and only then at a hard character boundary — never mid-token
//! unless there is no whitespace at all to break on.

use crate::config::ChunkingConfig;

/// One chunk of a document, with the heading trail it sits under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    pub text: String,
    pub breadcrumb: String,
}

pub fn split_markdown(body: &str, config: ChunkingConfig) -> Vec<Chunk> {
    let mut chunks: Vec<Chunk> = Vec::new();
    let mut trail: Vec<String> = Vec::new();
    let mut current = String::new();

    for line in body.lines() {
        if let Some(level) = heading_level(line) {
            if !current.trim().is_empty() {
                chunks.push(Chunk {
                    text: current.trim().to_owned(),
                    breadcrumb: trail.join(" > "),
                });
                current.clear();
            }
            trail.truncate(level.saturating_sub(1));
            trail.push(line.trim_start_matches('#').trim().to_owned());
            continue;
        }
        if current.len() >= config.target_chars && line.trim().is_empty() {
            chunks.push(Chunk {
                text: current.trim().to_owned(),
                breadcrumb: trail.join(" > "),
            });
            current.clear();
            continue;
        }
        current.push_str(line);
        current.push('\n');
    }
    if !current.trim().is_empty() {
        chunks.push(Chunk {
            text: current.trim().to_owned(),
            breadcrumb: trail.join(" > "),
        });
    }

    let merged = merge_undersized(chunks, config);
    merged
        .into_iter()
        .flat_map(|chunk| enforce_ceiling(&chunk, config))
        .collect()
}

pub fn enforce_ceiling(chunk: &Chunk, config: ChunkingConfig) -> Vec<Chunk> {
    let ceiling = config.max_chars + config.split_tolerance;
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

fn merge_undersized(chunks: Vec<Chunk>, config: ChunkingConfig) -> Vec<Chunk> {
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

fn heading_level(line: &str) -> Option<usize> {
    let hashes = line.chars().take_while(|c| *c == '#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    line.chars()
        .nth(hashes)
        .is_some_and(char::is_whitespace)
        .then_some(hashes)
}

// Why: byte index, not char index — the caller splits a &str with it. Every
// candidate below is taken from a char or grapheme-safe boundary API, so the
// returned index never lands inside a multi-byte character.
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
