//! The `none` strategy for Jira tickets.
//!
//! The `none` strategy for Jira: a ticket's description and its discussion
//! answer different questions, so the body stays one chunk, the comment
//! thread another, and a long thread is regrouped per comment so a hit
//! points at the exact turn instead of one oversized blob.

use super::Chunk;
use crate::config::ChunkingConfig;
use regex::Regex;
use std::sync::LazyLock;

pub const SOURCE_TYPE: &str = "jira";
pub const JIRA_SPLIT_THRESHOLD: usize = 5000;
pub const JIRA_MAX_GROUP: usize = 3000;

static COMMENT_START: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\*\*[^*\n]+\*\*\s+\(\d{4}-\d{2}-\d{2}\):")
        .unwrap_or_else(|e| unreachable!("comment regex is well-formed: {e}"))
});

#[must_use]
pub fn split(content: &str, config: ChunkingConfig) -> Vec<Chunk> {
    let Some(marker) = content.find("\n## Comments") else {
        return vec![Chunk::plain(content)];
    };
    let body = content[..marker].trim();
    let comments_raw = content[marker..].trim();
    let title = super::lines::infer_title(content, "");
    let header = format!("{title}\n\n## Comments\n\n");
    let comment_text = comments_raw.trim_start_matches("## Comments").trim();

    if body.chars().count() < config.min_chars || comment_text.is_empty() {
        return vec![Chunk::plain(content)];
    }
    if comments_raw.chars().count() <= JIRA_SPLIT_THRESHOLD {
        return vec![
            Chunk::plain(body),
            Chunk::plain(format!("{header}{comment_text}")),
        ];
    }

    let mut chunks = vec![Chunk::plain(body)];
    let mut current = String::new();
    for comment in comments(comment_text) {
        if !current.is_empty()
            && header.chars().count() + current.chars().count() + comment.chars().count()
                > JIRA_MAX_GROUP
        {
            chunks.push(Chunk::plain(format!("{header}{}", current.trim())));
            current.clear();
        }
        if !current.is_empty() {
            current.push_str("\n\n");
        }
        current.push_str(comment.trim());
    }
    if !current.trim().is_empty() {
        chunks.push(Chunk::plain(format!("{header}{}", current.trim())));
    }
    chunks
}

fn comments(text: &str) -> Vec<&str> {
    let starts: Vec<usize> = COMMENT_START.find_iter(text).map(|m| m.start()).collect();
    if starts.is_empty() {
        return vec![text];
    }
    let mut out = Vec::new();
    for (i, start) in starts.iter().enumerate() {
        let end = starts.get(i + 1).copied().unwrap_or(text.len());
        out.push(&text[*start..end]);
    }
    out
}
