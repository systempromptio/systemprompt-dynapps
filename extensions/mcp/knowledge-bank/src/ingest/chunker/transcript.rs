//! Transcript splitting by whole speaker turns.
//!
//! Transcript splitting: accumulate whole speaker turns up to `target_chars`,
//! never cutting mid-turn, and prepend a meeting header to every chunk. In a
//! transcript the natural unit is a speaker turn — splitting inside one
//! severs a statement from its speaker and its answer.

use super::Chunk;
use crate::config::ChunkingConfig;
use regex::Regex;
use std::sync::LazyLock;

pub const TRANSCRIPT_SPEAKER_MIN: usize = 5;

static SPEAKER_HEADER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?:\*\*)?[A-Z][\w.'-]*(?: [A-Z][\w.'-]*){0,3}(?:\*\*)?\s*(?:\(?\d{1,2}:\d{2}(?::\d{2})?\)?|:)\s*$",
    )
    .unwrap_or_else(|e| unreachable!("speaker regex is well-formed: {e}"))
});

static HEADER_DATE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b\d{1,2} (?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]* \d{4}\b")
        .unwrap_or_else(|e| unreachable!("date regex is well-formed: {e}"))
});

#[must_use]
pub fn is_speaker_header(line: &str) -> bool {
    line.len() <= 80 && SPEAKER_HEADER.is_match(line.trim())
}

#[must_use]
pub fn looks_like_transcript(body: &str) -> bool {
    body.lines().filter(|l| is_speaker_header(l)).count() >= TRANSCRIPT_SPEAKER_MIN
}

#[must_use]
pub fn meeting_header(title: &str, body: &str) -> String {
    if title.trim().is_empty() {
        return String::new();
    }
    let opening: String = body.chars().take(500).collect();
    HEADER_DATE.find(&opening).map_or_else(
        || format!("Meeting: {title}"),
        |date| format!("Meeting: {title} | Date: {}", date.as_str()),
    )
}

#[must_use]
pub fn split_script(body: &str, title: &str, config: ChunkingConfig) -> Vec<Chunk> {
    let header = meeting_header(title, body);
    let turns = turns(body);
    let mut chunks = Vec::new();
    let mut current = String::new();
    for turn in turns {
        if !current.is_empty()
            && current.chars().count() + turn.chars().count() > config.target_chars
        {
            chunks.push(with_header(&header, &current));
            current.clear();
        }
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(&turn);
    }
    if !current.trim().is_empty() {
        chunks.push(with_header(&header, &current));
    }
    chunks
}

fn with_header(header: &str, text: &str) -> Chunk {
    if header.is_empty() {
        Chunk::plain(text.trim())
    } else {
        Chunk::plain(format!("{header}\n\n{}", text.trim()))
    }
}

fn turns(body: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut previous_blank = true;
    for line in body.lines() {
        let boundary = is_speaker_header(line) || (previous_blank && !line.trim().is_empty());
        if boundary && !current.trim().is_empty() {
            out.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(line);
        previous_blank = line.trim().is_empty();
    }
    if !current.trim().is_empty() {
        out.push(current);
    }
    out
}
