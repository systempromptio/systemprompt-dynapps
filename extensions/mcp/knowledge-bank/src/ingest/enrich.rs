//! Index-time contextual enrichment of chunks.
//!
//! Index-time contextual enrichment: one or two factual sentences prepended
//! to a chunk so it is self-contained for search. A mid-document chunk often
//! lacks the terms someone would search for — the feature name, the ticket
//! key, the section it belongs to. The model may only add facts visible in
//! the document; on any failure the chunk is used unchanged, and line
//! pointers are always computed against the pre-enrichment text.

use crate::model::ModelProvider;

const CONTEXT_CHARS: usize = 15_000;
const MAX_PREFIX_CHARS: usize = 400;

const ENRICH_SYSTEM: &str = "You add retrieval context to a chunk of a document. Given the \
    document's title, heading outline and opening, and one chunk, write one or two short factual \
    sentences that say what the chunk is about and where it sits in the document, using only \
    facts visible in the material. Do not infer, summarise beyond the chunk, or repeat the \
    chunk. Reply with the sentences only.";

/// The document-level material every enrichment call shares.
#[derive(Debug, Clone)]
pub struct DocumentContext {
    pub title: String,
    pub outline: String,
    pub opening: String,
}

impl DocumentContext {
    #[must_use]
    pub fn build(title: &str, body: &str) -> Self {
        let outline = body
            .lines()
            .filter_map(|line| super::chunker::markdown::heading(line))
            .map(|(level, text)| format!("{}{text}", "  ".repeat(level.saturating_sub(1))))
            .collect::<Vec<_>>()
            .join("\n");
        Self {
            title: title.to_owned(),
            outline,
            opening: body.chars().take(CONTEXT_CHARS).collect(),
        }
    }
}

pub async fn enrich_chunk(
    model: &dyn ModelProvider,
    context: &DocumentContext,
    chunk: &str,
) -> Option<String> {
    let user = format!(
        "TITLE:\n{}\n\nOUTLINE:\n{}\n\nOPENING:\n{}\n\nCHUNK:\n{chunk}",
        context.title, context.outline, context.opening
    );
    match model.converse(ENRICH_SYSTEM, &user).await {
        Ok(reply) => {
            let prefix: String = reply.trim().chars().take(MAX_PREFIX_CHARS).collect();
            (!prefix.is_empty()).then_some(prefix)
        },
        Err(e) => {
            tracing::warn!(error = %e, "enrichment failed; chunk used unchanged");
            None
        },
    }
}
