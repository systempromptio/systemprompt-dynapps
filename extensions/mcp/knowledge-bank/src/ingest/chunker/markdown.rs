//! The `script` strategy for structured Markdown: split along the heading
//! outline, prefix each chunk with its heading breadcrumb, and break wide
//! tables row by row while repeating the header block.
//!
//! A chunk lifted out of a page loses its place; the breadcrumb (`H1 > H2`)
//! re-attaches that context so both the embedding and the reader know a
//! "Cards" section lives under "Checkout > Payment". Repeating a table's
//! header on every row-chunk keeps each fragment interpretable on its own.

use super::Chunk;
use super::size::split_by_sentences;
use crate::config::ChunkingConfig;

struct Section {
    crumbs: Vec<String>,
    body: String,
}

#[must_use]
pub fn split(body: &str, config: ChunkingConfig) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    for section in sections(body) {
        let breadcrumb = if section.crumbs.len() > 1 {
            section.crumbs.join(" > ")
        } else {
            String::new()
        };
        let text = section.body.trim();
        if text.is_empty() {
            continue;
        }
        if text.chars().count() + breadcrumb.chars().count() <= config.max_chars {
            chunks.push(Chunk {
                text: text.to_owned(),
                breadcrumb,
            });
            continue;
        }
        for piece in split_blocks(text, config) {
            chunks.push(Chunk {
                text: piece,
                breadcrumb: breadcrumb.clone(),
            });
        }
    }
    chunks
}

fn sections(body: &str) -> Vec<Section> {
    let mut out: Vec<Section> = Vec::new();
    let mut crumbs: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_fence = false;

    for line in body.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
        }
        if !in_fence && let Some((level, title)) = heading(line) {
            if current.trim().is_empty() {
                current.clear();
            } else {
                out.push(Section {
                    crumbs: crumbs.clone(),
                    body: std::mem::take(&mut current),
                });
            }
            crumbs.truncate(level.saturating_sub(1));
            while crumbs.len() < level.saturating_sub(1) {
                crumbs.push(String::new());
            }
            crumbs.push(title.to_owned());
            crumbs.retain(|c| !c.is_empty());
        }
        current.push_str(line);
        current.push('\n');
    }
    if !current.trim().is_empty() {
        out.push(Section {
            crumbs,
            body: current,
        });
    }
    out
}

#[must_use]
pub fn heading(line: &str) -> Option<(usize, &str)> {
    let hashes = line.chars().take_while(|c| *c == '#').count();
    if hashes == 0 || hashes > 4 {
        return None;
    }
    let rest = &line[hashes..];
    if !rest.starts_with(' ') {
        return None;
    }
    let title = rest.trim().trim_end_matches('#').trim();
    (!title.is_empty()).then_some((hashes, title))
}

fn is_table_line(line: &str) -> bool {
    let t = line.trim();
    t.starts_with('|') && t.ends_with('|')
}

fn is_table_separator(line: &str) -> bool {
    let t = line.trim();
    is_table_line(t) && t.chars().all(|c| matches!(c, '|' | '-' | ':' | ' '))
}

fn split_blocks(text: &str, config: ChunkingConfig) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for block in text.split("\n\n") {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }
        let is_table = block.lines().any(is_table_separator);
        if is_table {
            flush(&mut current, &mut out);
            if block.chars().count() <= config.max_chars {
                out.push(block.to_owned());
            } else {
                out.extend(split_table_rows(block, config.max_chars));
            }
            continue;
        }
        if block.chars().count() > config.max_chars {
            flush(&mut current, &mut out);
            out.extend(split_by_sentences(block, config.target_chars));
            continue;
        }
        if !current.is_empty()
            && current.chars().count() + block.chars().count() > config.target_chars
        {
            flush(&mut current, &mut out);
        }
        if !current.is_empty() {
            current.push_str("\n\n");
        }
        current.push_str(block);
    }
    flush(&mut current, &mut out);
    out
}

fn flush(current: &mut String, out: &mut Vec<String>) {
    if current.trim().is_empty() {
        current.clear();
    } else {
        out.push(std::mem::take(current));
    }
}

#[must_use]
pub fn split_table_rows(block: &str, max_chars: usize) -> Vec<String> {
    let lines: Vec<&str> = block.lines().collect();
    let Some(sep) = lines.iter().position(|l| is_table_separator(l)) else {
        return vec![block.to_owned()];
    };
    let header = lines[..=sep].join("\n");
    let mut out = Vec::new();
    let mut current = header.clone();
    for row in &lines[sep + 1..] {
        if current.chars().count() + row.chars().count() + 1 > max_chars && current != header {
            out.push(std::mem::replace(&mut current, header.clone()));
        }
        current.push('\n');
        current.push_str(row);
    }
    if current != header {
        out.push(current);
    }
    out
}
