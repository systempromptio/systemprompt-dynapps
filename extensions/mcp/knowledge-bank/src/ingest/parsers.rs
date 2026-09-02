//! Binary and text parsers: PDF and DOCX become plain text, Markdown and
//! text pass through, everything else is unsupported by design.
//!
//! `dewrap_transcript` repairs PDF line-wrapping in meeting transcripts —
//! speaker headers on their own line, one sentence per line — changing only
//! whitespace, never wording, so line pointers into the result stay honest.

use crate::error::KnowledgeBankError;
use std::io::{Cursor, Read};

pub const SUPPORTED_SUFFIXES: [&str; 4] = ["md", "txt", "pdf", "docx"];

#[must_use]
pub fn is_supported(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| SUPPORTED_SUFFIXES.contains(&e.to_ascii_lowercase().as_str()))
}

#[must_use]
pub fn suffix_of(file_name: &str) -> String {
    std::path::Path::new(file_name)
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default()
}

pub fn parse_bytes(path: &std::path::Path, bytes: &[u8]) -> Result<String, KnowledgeBankError> {
    parse_named(&path.to_string_lossy(), bytes)
}

pub fn parse_named(file_name: &str, bytes: &[u8]) -> Result<String, KnowledgeBankError> {
    let suffix = suffix_of(file_name);
    match suffix.as_str() {
        "pdf" => parse_pdf(bytes),
        "docx" => parse_docx(bytes),
        "md" | "txt" => Ok(String::from_utf8_lossy(bytes).into_owned()),
        other => Err(KnowledgeBankError::Internal(format!(
            "unsupported file type .{other}"
        ))),
    }
}

pub fn parse_pdf(bytes: &[u8]) -> Result<String, KnowledgeBankError> {
    pdf_extract::extract_text_from_mem(bytes)
        // Why: OutputError mixes I/O, font, and content-stream failures; none
        // is actionable beyond "this PDF could not be read".
        // lint-ok: error-adapt
        .map_err(|e| KnowledgeBankError::Internal(format!("pdf: {e}")))
}

pub fn parse_docx(bytes: &[u8]) -> Result<String, KnowledgeBankError> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        // Why: ZipError is variant-less for our purposes; a DOCX that cannot
        // be opened is one per-document failure whatever the cause.
        // lint-ok: error-adapt
        .map_err(|e| KnowledgeBankError::Internal(format!("docx: {e}")))?;
    let mut file = archive
        .by_name("word/document.xml")
        // Why: same adaptation as above.
        // lint-ok: error-adapt
        .map_err(|e| KnowledgeBankError::Internal(format!("docx: {e}")))?;
    let mut xml = String::new();
    file.read_to_string(&mut xml)
        // Why: io::Error here is a corrupt archive member, same failure class.
        // lint-ok: error-adapt
        .map_err(|e| KnowledgeBankError::Internal(format!("docx: {e}")))?;
    Ok(docx_xml_to_text(&xml))
}

#[must_use]
pub fn docx_xml_to_text(xml: &str) -> String {
    let mut out = String::with_capacity(xml.len() / 4);
    let mut rest = xml;
    while let Some(start) = rest.find('<') {
        out.push_str(&rest[..start]);
        let Some(end) = rest[start..].find('>') else {
            break;
        };
        let tag = &rest[start + 1..start + end];
        if tag.starts_with("/w:p") || tag.starts_with("w:br") {
            out.push('\n');
        } else if tag.starts_with("w:tab") {
            out.push('\t');
        }
        rest = &rest[start + end + 1..];
    }
    out.push_str(rest);
    unescape_xml(&out)
}

fn unescape_xml(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

#[must_use]
pub fn dewrap_transcript(text: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut buffer = String::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            flush_sentences(&mut buffer, &mut out);
            continue;
        }
        if super::chunker::transcript::is_speaker_header(line) {
            flush_sentences(&mut buffer, &mut out);
            out.push(line.to_owned());
            continue;
        }
        if !buffer.is_empty() {
            buffer.push(' ');
        }
        buffer.push_str(line);
    }
    flush_sentences(&mut buffer, &mut out);
    out.join("\n")
}

fn flush_sentences(buffer: &mut String, out: &mut Vec<String>) {
    if buffer.trim().is_empty() {
        buffer.clear();
        return;
    }
    let mut sentence = String::new();
    for word in buffer.split_whitespace() {
        if !sentence.is_empty() {
            sentence.push(' ');
        }
        sentence.push_str(word);
        if word.ends_with(['.', '!', '?']) {
            out.push(std::mem::take(&mut sentence));
        }
    }
    if !sentence.is_empty() {
        out.push(sentence);
    }
    buffer.clear();
}
