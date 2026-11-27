//! The `asset://<id>` references of a Markdown body.
//!
//! The stored text names a file by its id and never by a URL: `get` and
//! `export` add signed URLs beside the text, and the indexer replaces each
//! reference by its alternative text, so no URL ever reaches a chunk or an
//! embedding.
//!
//! `references` lists the files a body names, each once, in the order they
//! first appear. `index_text` is the body as the indexer reads it:
//! `![alt](asset://1)` and `[text](asset://1)` become their text, and any
//! other `asset://` reference is dropped.

use crate::ids::AssetId;

pub const SCHEME: &str = "asset://";

pub fn reference(asset: AssetId) -> String {
    format!("{SCHEME}{asset}")
}

pub fn references(markdown: &str) -> Vec<AssetId> {
    let mut found = Vec::new();
    for (at, _) in markdown.match_indices(SCHEME) {
        let digits: String = markdown[at + SCHEME.len()..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        let Ok(id) = digits.parse::<i64>() else {
            continue;
        };
        if !found.contains(&AssetId(id)) {
            found.push(AssetId(id));
        }
    }
    found
}

pub fn index_text(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut rest = markdown;
    let opener = format!("]({SCHEME}");
    while let Some(at) = rest.find(&opener) {
        let before = &rest[..at];
        let after = &rest[at + 2..];
        let Some(close) = after.find(')') else {
            break;
        };
        let Some(open) = before.rfind('[') else {
            out.push_str(before);
            rest = after;
            continue;
        };
        let start = if before[..open].ends_with('!') {
            open - 1
        } else {
            open
        };
        out.push_str(&before[..start]);
        out.push_str(&before[open + 1..]);
        rest = &after[close + 1..];
    }
    out.push_str(rest);
    without_bare(&out)
}

fn without_bare(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(SCHEME) {
        out.push_str(&rest[..at]);
        let tail = &rest[at + SCHEME.len()..];
        let digits = tail.chars().take_while(char::is_ascii_digit).count();
        rest = &tail[digits..];
    }
    out.push_str(rest);
    out
}
