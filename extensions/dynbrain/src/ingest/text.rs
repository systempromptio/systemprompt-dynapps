//! The text rules of Python's `str` that dynbrain's chunker is written
//! against, so the port cuts exactly where it does.
//!
//! Whitespace is Python's (`str.isspace`, which also counts the four
//! information separators), a line ends at any of the breaks `splitlines`
//! knows, the paragraph break is `\n\s*\n` and a sentence ends at the
//! whitespace that follows `.`, `!`, `?` or their full-width forms. Lengths
//! are in characters, not bytes.

pub fn is_space(c: char) -> bool {
    c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}')
}

pub fn strip(text: &str) -> &str {
    text.trim_matches(is_space)
}

pub fn word_count(text: &str) -> usize {
    text.split(is_space).filter(|word| !word.is_empty()).count()
}

pub fn splitlines(text: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        if !is_line_break(c) {
            continue;
        }
        lines.push(&text[start..at]);
        start = at + c.len_utf8();
        if c == '\r' && chars.next_if(|&(_, next)| next == '\n').is_some() {
            start += 1;
        }
    }
    if start < text.len() {
        lines.push(&text[start..]);
    }
    lines
}

pub fn paragraphs(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut at = 0;
    while at < text.len() {
        if text.as_bytes()[at] == b'\n'
            && let Some(end) = paragraph_break_end(text, at)
        {
            out.push(&text[start..at]);
            start = end;
            at = end;
            continue;
        }
        at += 1;
    }
    out.push(&text[start..]);
    out
}

pub fn sentences(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut last: Option<char> = None;
    let mut run: Option<usize> = None;
    for (at, c) in text.char_indices() {
        if is_space(c) {
            run.get_or_insert(at);
            continue;
        }
        if let Some(from) = run.take()
            && last.is_some_and(is_terminator)
        {
            push_trimmed(&mut out, &text[start..from]);
            start = at;
        }
        last = Some(c);
    }
    push_trimmed(&mut out, &text[start..]);
    out
}

pub fn byte_index_of_char(text: &str, chars: usize) -> usize {
    text.char_indices()
        .nth(chars)
        .map_or(text.len(), |(at, _)| at)
}

const fn is_line_break(c: char) -> bool {
    matches!(
        c,
        '\n' | '\r'
            | '\u{b}'
            | '\u{c}'
            | '\u{1c}'
            | '\u{1d}'
            | '\u{1e}'
            | '\u{85}'
            | '\u{2028}'
            | '\u{2029}'
    )
}

const fn is_terminator(c: char) -> bool {
    matches!(c, '.' | '!' | '?' | '。' | '！' | '？')
}

fn paragraph_break_end(text: &str, newline: usize) -> Option<usize> {
    let after = newline + 1;
    let run = text[after..]
        .char_indices()
        .find(|&(_, c)| !is_space(c))
        .map_or(text.len(), |(at, _)| after + at);
    text[after..run].rfind('\n').map(|last| after + last + 1)
}

fn push_trimmed<'a>(out: &mut Vec<&'a str>, piece: &'a str) {
    let piece = strip(piece);
    if !piece.is_empty() {
        out.push(piece);
    }
}
