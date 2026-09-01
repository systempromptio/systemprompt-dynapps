//! Chunk metadata: title and `start-end` line range.
//!
//! Chunk metadata: the document title and the `start-end` line range each
//! chunk occupies in the original text. Pointers are computed against the
//! pre-enrichment source, so they stay true even though the stored text may
//! carry a breadcrumb or context prefix.

const PROBE_CHARS: usize = 80;

#[must_use]
pub fn infer_title(body: &str, fallback: &str) -> String {
    if let Some(title) = frontmatter_title(body) {
        return title;
    }
    for line in body.lines() {
        if let Some((_, title)) = super::markdown::heading(line) {
            return title.to_owned();
        }
    }
    fallback.to_owned()
}

fn frontmatter_title(body: &str) -> Option<String> {
    let rest = body.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    rest[..end]
        .lines()
        .find_map(|line| line.strip_prefix("title:"))
        .map(|v| v.trim().trim_matches('"').to_owned())
        .filter(|v| !v.is_empty())
}

// Why: returns the 1-based inclusive range and the byte offset to continue
// from, so a repeated opening line maps to its own occurrence, not the first.
#[must_use]
pub fn find_line_range(original: &str, chunk: &str, cursor: usize) -> Option<(String, usize)> {
    let probe_line = chunk.lines().map(str::trim).find(|l| !l.is_empty())?;
    let probe: String = probe_line.chars().take(PROBE_CHARS).collect();
    let haystack = original.get(cursor..)?;
    let found = haystack.find(&probe)? + cursor;
    let start_line = original[..found].matches('\n').count() + 1;
    let chunk_lines = chunk.trim().lines().count().max(1);
    let end_line = start_line + chunk_lines - 1;
    let end_offset = original[found..]
        .char_indices()
        .filter(|(_, c)| *c == '\n')
        .nth(chunk_lines - 1)
        .map_or(original.len(), |(i, _)| found + i);
    Some((format!("{start_line}-{end_line}"), end_offset))
}
