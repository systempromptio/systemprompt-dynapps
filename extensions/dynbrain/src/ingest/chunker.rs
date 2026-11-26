//! The chunker, a port of dynbrain's `chunk_document`.
//!
//! The text is read as Markdown, split into sentences (paragraph breaks count
//! as sentence breaks) and grouped greedily: a chunk closes when it holds
//! [`TARGET_WORDS`] words of new text, and the next one starts with the last
//! sentences of the previous up to [`OVERLAP_WORDS`] words. Headings never
//! decide where a chunk ends; they only say what it is about: each chunk
//! carries the heading trail most of its words were written under
//! ([`Chunk::heading_path`], empty when the document has no headings). A
//! chunk is never longer than [`MAX_CHARS`] characters.
//!
//! The results match the Python's for the same text: the unit tests compare
//! them with chunks Python produced from documents of dynbrain.

use super::text::{byte_index_of_char, paragraphs, sentences, splitlines, strip, word_count};

pub const TARGET_WORDS: usize = 300;
pub const OVERLAP_WORDS: usize = 50;
pub const MAX_CHARS: usize = 6000;
pub const HEADING_SEPARATOR: &str = " > ";

/// One piece of a document and the heading trail it sits under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    pub content: String,
    pub heading_path: String,
}

type Unit<'a> = (&'a str, &'a str);

pub fn chunk_document(text: &str) -> Vec<Chunk> {
    let sections = split_sections(text);
    let units: Vec<Unit<'_>> = sections
        .iter()
        .flat_map(|(path, section)| {
            paragraphs(section)
                .into_iter()
                .flat_map(sentences)
                .map(move |sentence| (sentence, path.as_str()))
        })
        .collect();
    group(&units, TARGET_WORDS, OVERLAP_WORDS)
        .iter()
        .flat_map(|members| {
            let heading_path = dominant_path(members);
            let content = members
                .iter()
                .map(|&(sentence, _)| sentence)
                .collect::<Vec<_>>()
                .join(" ");
            cap_chars(content, MAX_CHARS)
                .into_iter()
                .map(move |piece| Chunk {
                    content: piece,
                    heading_path: heading_path.to_owned(),
                })
        })
        .collect()
}

pub fn split_sections(text: &str) -> Vec<(String, String)> {
    let lines = splitlines(text);
    let mut builder = Sections::default();
    let mut stack: Vec<(usize, &str)> = Vec::new();
    let mut in_fence = false;
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        if is_fence(line) {
            in_fence = !in_fence;
            builder.body.push(line);
            index += 1;
            continue;
        }
        let heading = if in_fence {
            None
        } else {
            heading_at(&lines, index)
        };
        let Some((level, title, consumed)) = heading else {
            builder.body.push(line);
            index += 1;
            continue;
        };
        builder.flush();
        while stack.last().is_some_and(|&(open, _)| open >= level) {
            stack.pop();
        }
        stack.push((level, title));
        builder.path = stack
            .iter()
            .map(|&(_, title)| title)
            .filter(|title| !title.is_empty())
            .collect::<Vec<_>>()
            .join(HEADING_SEPARATOR);
        builder.head = lines[index..index + consumed].to_vec();
        index += consumed;
    }
    builder.finish()
}

#[derive(Default)]
struct Sections<'a> {
    out: Vec<(String, String)>,
    carried: Vec<&'a str>,
    head: Vec<&'a str>,
    body: Vec<&'a str>,
    path: String,
}

impl<'a> Sections<'a> {
    fn flush(&mut self) {
        if self.body.iter().any(|line| !strip(line).is_empty()) {
            self.out.push((self.path.clone(), self.text()));
            self.carried.clear();
        } else {
            self.carried.append(&mut self.head);
            self.carried.append(&mut self.body);
        }
        self.head.clear();
        self.body.clear();
    }

    fn finish(mut self) -> Vec<(String, String)> {
        let section = self.text();
        if !section.is_empty() {
            self.out.push((self.path, section));
        }
        self.out
    }

    fn text(&self) -> String {
        let all: Vec<&'a str> = self
            .carried
            .iter()
            .chain(&self.head)
            .chain(&self.body)
            .copied()
            .collect();
        strip(&all.join("\n")).to_owned()
    }
}

fn heading_at<'a>(lines: &[&'a str], index: usize) -> Option<(usize, &'a str, usize)> {
    let line = lines[index];
    if let Some((level, title)) = atx(line) {
        return Some((level, title, 1));
    }
    let underline = lines.get(index + 1)?;
    let level = setext_level(underline)?;
    (!strip(line).is_empty() && !is_list_like(line)).then(|| (level, strip(line), 2))
}

fn atx(line: &str) -> Option<(usize, &str)> {
    let hashes = line.bytes().take_while(|&b| b == b'#').count();
    let rest = line[hashes..].strip_prefix([' ', '\t'])?;
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let title = rest
        .trim_start_matches([' ', '\t'])
        .trim_end_matches([' ', '\t'])
        .trim_end_matches('#')
        .trim_end_matches([' ', '\t']);
    Some((hashes, strip(title)))
}

fn setext_level(line: &str) -> Option<usize> {
    let first = line.chars().next()?;
    if first != '=' && first != '-' {
        return None;
    }
    let tail = line.trim_start_matches(first);
    let underline_only = tail.chars().all(|c| c == ' ' || c == '\t');
    (underline_only && strip(line).chars().count() >= 2).then_some(if first == '=' { 1 } else { 2 })
}

fn is_fence(line: &str) -> bool {
    let rest = line.trim_start_matches([' ', '\t']);
    rest.starts_with("```") || rest.starts_with("~~~")
}

fn is_list_like(line: &str) -> bool {
    let rest = line.trim_start_matches([' ', '\t']);
    let after = if rest.starts_with(['-', '*', '+', '>', '|']) {
        &rest[1..]
    } else {
        let digits = rest.trim_start_matches(|c: char| c.is_ascii_digit());
        if digits.len() == rest.len() || !digits.starts_with(['.', ')']) {
            return false;
        }
        &digits[1..]
    };
    after.starts_with([' ', '\t'])
}

fn group<'a>(units: &[Unit<'a>], target: usize, overlap_words: usize) -> Vec<Vec<Unit<'a>>> {
    let mut groups = Vec::new();
    let mut current: Vec<Unit<'a>> = Vec::new();
    let mut new_words = 0;
    for &unit in units {
        current.push(unit);
        new_words += word_count(unit.0);
        if new_words < target {
            continue;
        }
        let mut overlap = Vec::new();
        let mut words = 0;
        for &item in current.iter().rev() {
            overlap.push(item);
            words += word_count(item.0);
            if words >= overlap_words {
                break;
            }
        }
        overlap.reverse();
        groups.push(std::mem::replace(&mut current, overlap));
        new_words = 0;
    }
    if new_words > 0 {
        groups.push(current);
    }
    groups
}

fn dominant_path<'a>(members: &[Unit<'a>]) -> &'a str {
    let mut weights: Vec<(&str, usize)> = Vec::new();
    for &(sentence, path) in members {
        let words = word_count(sentence);
        match weights.iter_mut().find(|(known, _)| *known == path) {
            Some((_, total)) => *total += words,
            None => weights.push((path, words)),
        }
    }
    let mut best: Option<(&str, usize)> = None;
    for &(path, total) in &weights {
        if best.is_none_or(|(_, top)| total > top) {
            best = Some((path, total));
        }
    }
    best.map_or("", |(path, _)| path)
}

fn cap_chars(chunk: String, max_chars: usize) -> Vec<String> {
    if chunk.chars().count() <= max_chars {
        return vec![chunk];
    }
    let mut out = Vec::new();
    let mut current = String::new();
    for word in chunk.split(' ') {
        let mut word = word;
        while word.chars().count() > max_chars {
            if !current.is_empty() {
                out.push(std::mem::take(&mut current));
            }
            let cut = byte_index_of_char(word, max_chars);
            out.push(word[..cut].to_owned());
            word = &word[cut..];
        }
        let candidate = strip(&format!("{current} {word}")).to_owned();
        if candidate.chars().count() > max_chars && !current.is_empty() {
            out.push(std::mem::replace(&mut current, word.to_owned()));
        } else {
            current = candidate;
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}
