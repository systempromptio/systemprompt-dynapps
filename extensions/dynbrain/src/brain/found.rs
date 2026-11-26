//! What `search` and `answer` answer.
//!
//! The field names are those of `docs_search` and of `docs_answer` with
//! `synthesize: false`.
//!
//! `answer` hands back the assembled context, never an answer: no model is
//! called. The caller's model writes it from `context`, cites `[n]` against
//! `citations`, and says "not documented" when `confidence` is `low`.

use serde::Serialize;

use crate::ids::{ArticleId, SpaceId};
use crate::search::confidence::ConfidenceVerdict;
use crate::search::service::RetrievalMode;

/// One result of `search`: a passage of a document, titled by the variant it
/// was found in.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SearchHit {
    pub article_id: ArticleId,
    pub space: SpaceId,
    pub path: String,
    pub title: String,
    pub description: String,
    pub lang: String,
    pub snippet: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
}

/// What `search` answers.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SearchResults {
    pub query: String,
    pub mode: RetrievalMode,
    pub results: Vec<SearchHit>,
    pub total: usize,
    #[serde(flatten)]
    pub verdict: ConfidenceVerdict,
}

/// A source of an answer's context, numbered by its position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Citation {
    pub path: String,
    pub title: String,
    pub lang: String,
    pub snippet: String,
}

/// What `answer` hands back: always the context, never a written answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AnswerMode {
    Context,
}

/// What `answer` answers.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Answer {
    pub context: String,
    pub citations: Vec<Citation>,
    pub mode: AnswerMode,
    #[serde(flatten)]
    pub verdict: ConfidenceVerdict,
}
