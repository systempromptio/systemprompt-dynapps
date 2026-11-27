//! The context a chunk is embedded with, and the synopsis that can be part
//! of it.
//!
//! A chunk cut out of its document loses what the document knew about it, so
//! the embedding is made from a prefix plus the chunk: the document's title
//! and description, then whatever locates the chunk inside it, by strategy:
//! the heading path, a one-sentence synopsis a model wrote, both, or the
//! heading path when there is one and the synopsis when there is not (`auto`,
//! the default). The prefix goes only into the embedding; the stored chunk,
//! the text-search vector and every snippet stay the clean content. A chunk
//! that is mostly fenced code gets no prefix: prose around a snippet drags
//! its vector away from the code someone searches for.

use sha2::{Digest, Sha256};

use super::text::{byte_index_of_char, strip};

const CODE_CHUNK_TENTHS: usize = 7;
pub const SYNOPSIS_MAX_WORDS: usize = 30;
pub const SYNOPSIS_DOC_CHARS: usize = 8000;
const TRUNCATED_MARK: &str = "\n[...truncated...]";
const SAME_LANGUAGE: &str = "the SAME language as the document";

/// How a chunk is situated inside its document before it is embedded.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ContextStrategy {
    #[default]
    Auto,
    Heading,
    Llm,
    HeadingLlm,
    Document,
    Off,
}

impl ContextStrategy {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "auto" => Some(Self::Auto),
            "heading" => Some(Self::Heading),
            "llm" => Some(Self::Llm),
            "heading_llm" => Some(Self::HeadingLlm),
            "document" => Some(Self::Document),
            "off" => Some(Self::Off),
            _ => None,
        }
    }

    pub const fn wants_every_synopsis(self) -> bool {
        matches!(self, Self::Llm | Self::HeadingLlm)
    }

    pub fn needs_synopsis(self, heading_path: &str, chat_available: bool) -> bool {
        chat_available
            && (self.wants_every_synopsis() || (self == Self::Auto && heading_path.is_empty()))
    }
}

/// The parts of a chunk the context is made of.
#[derive(Debug, Clone, Copy)]
pub struct ChunkContext<'a> {
    pub content: &'a str,
    pub heading_path: &'a str,
    pub synopsis: Option<&'a str>,
}

/// The two messages that ask a model to situate an excerpt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynopsisPrompt {
    pub system: String,
    pub user: String,
}

pub fn document_context(title: &str, description: &str) -> String {
    join_lines(&[title, description])
}

pub fn prefix(strategy: ContextStrategy, document: &str, chunk: &ChunkContext<'_>) -> String {
    if strategy == ContextStrategy::Off || is_code_chunk(chunk.content) {
        return String::new();
    }
    let layer = layer(strategy, chunk);
    join_lines(&[document, layer.as_str()])
}

pub fn embedding_text(prefix: &str, content: &str) -> String {
    join_lines(&[prefix, content])
}

pub fn text_hash(text: &str) -> String {
    hex::encode(Sha256::digest(text.as_bytes()))
}

pub fn is_code_chunk(content: &str) -> bool {
    let sizes: Vec<usize> = content
        .split("```")
        .map(|segment| strip(segment).chars().count())
        .collect();
    if sizes.len() < 3 {
        return false;
    }
    let fenced: usize = sizes.iter().skip(1).step_by(2).sum();
    let total: usize = sizes.iter().sum();
    total > 0 && fenced * 10 >= total * CODE_CHUNK_TENTHS
}

pub fn synopsis_prompt(lang: &str, document: &str, excerpt: &str) -> SynopsisPrompt {
    let document = if document.chars().count() > SYNOPSIS_DOC_CHARS {
        let cut = byte_index_of_char(document, SYNOPSIS_DOC_CHARS);
        format!("{}{TRUNCATED_MARK}", &document[..cut])
    } else {
        document.to_owned()
    };
    SynopsisPrompt {
        system: format!(
            "You situate an excerpt inside the document it was taken from, so that the excerpt \
             can be found on its own by a search engine. Answer with ONE sentence of at most \
             {SYNOPSIS_MAX_WORDS} words, written in {}. Name the concrete subject, entities and \
             topic the excerpt covers, using the words someone would type when searching for it. \
             Do not summarise the whole document, do not explain, do not add a preamble: return \
             the sentence only.",
            language_name(lang)
        ),
        user: format!("<document>\n{document}\n</document>\n\n<excerpt>\n{excerpt}\n</excerpt>"),
    }
}

pub fn clean_synopsis(text: &str) -> String {
    strip(text)
        .trim_matches('"')
        .trim_matches('\'')
        .split(super::text::is_space)
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn layer(strategy: ContextStrategy, chunk: &ChunkContext<'_>) -> String {
    let synopsis = chunk.synopsis.unwrap_or_default();
    match strategy {
        ContextStrategy::Off | ContextStrategy::Document => String::new(),
        ContextStrategy::Llm => synopsis.to_owned(),
        ContextStrategy::HeadingLlm => join_lines(&[chunk.heading_path, synopsis]),
        ContextStrategy::Auto if chunk.heading_path.is_empty() => synopsis.to_owned(),
        ContextStrategy::Heading | ContextStrategy::Auto => chunk.heading_path.to_owned(),
    }
}

fn join_lines(parts: &[&str]) -> String {
    parts
        .iter()
        .filter(|part| !part.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join("\n")
}

fn language_name(code: &str) -> String {
    if code.is_empty() {
        return SAME_LANGUAGE.to_owned();
    }
    let base = code.split(['_', '-']).next().unwrap_or_default();
    let name = match base.to_ascii_lowercase().as_str() {
        "es" => "Spanish",
        "en" => "English",
        "nl" => "Dutch",
        "fr" => "French",
        "de" => "German",
        "pt" => "Portuguese",
        "it" => "Italian",
        _ => return code.to_owned(),
    };
    format!("{name} ({code})")
}
