//! Chunking: one document in, retrieval-sized chunk records out.
//!
//! Ported from the Node "project-context" chunker. The strategy is fixed per
//! source category — `llm` for transcripts, `script` for structured
//! Markdown, `none` for Jira tickets — and every path then runs through the
//! same size discipline: fragments under `min_chars` merge into a neighbour,
//! and nothing leaves longer than `max_chars + split_tolerance`. The LLM
//! never rewrites text; it only proposes boundaries, and every model failure
//! falls back to the deterministic splitter.

pub mod jira;
pub mod lines;
pub mod llm;
pub mod markdown;
pub mod size;
pub mod transcript;

pub use lines::{find_line_range, infer_title};
pub use size::enforce_ceiling;

use crate::config::{ChunkStrategy, ChunkingConfig};
use crate::model::ModelProvider;

/// One chunk of a document, with the heading trail it sits under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    pub text: String,
    pub breadcrumb: String,
}

impl Chunk {
    #[must_use]
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            breadcrumb: String::new(),
        }
    }
}

/// Everything the splitter needs to know about the document.
#[derive(Debug, Clone, Copy)]
pub struct SplitRequest<'a> {
    pub body: &'a str,
    pub title: &'a str,
    pub source_type: &'a str,
    pub strategy: ChunkStrategy,
    pub config: ChunkingConfig,
}

#[must_use]
pub fn split_markdown(body: &str, config: ChunkingConfig) -> Vec<Chunk> {
    size::discipline(markdown::split(body, config), config)
}

pub async fn split_document(
    request: SplitRequest<'_>,
    llm: Option<&dyn ModelProvider>,
) -> Vec<Chunk> {
    let raw = match request.strategy {
        ChunkStrategy::None => {
            if request.source_type == jira::SOURCE_TYPE {
                jira::split(request.body, request.config)
            } else {
                vec![Chunk::plain(request.body)]
            }
        },
        ChunkStrategy::Script => split_script(request),
        ChunkStrategy::Llm => split_llm(request, llm).await,
    };
    size::discipline(raw, request.config)
}

fn split_script(request: SplitRequest<'_>) -> Vec<Chunk> {
    if transcript::looks_like_transcript(request.body) {
        transcript::split_script(request.body, request.title, request.config)
    } else {
        markdown::split(request.body, request.config)
    }
}

async fn split_llm(request: SplitRequest<'_>, llm: Option<&dyn ModelProvider>) -> Vec<Chunk> {
    if !transcript::looks_like_transcript(request.body) {
        return markdown::split(request.body, request.config);
    }
    if let Some(model) = llm
        && let Some(chunks) = llm::split(model, request).await
    {
        return chunks;
    }
    transcript::split_script(request.body, request.title, request.config)
}
