//! Whether the corpus looks like it answers the question. It never filters.
//!
//! `high` and `low` split the reranker's score of the best result at the
//! threshold (0.70 by default; 0 switches the signal off and answers `high`).
//! When the reranker did not score the results their scores come from the
//! fusion, two orders of magnitude lower, so the verdict is `unrated`: the
//! tool cannot judge, which is not the same as the answer missing. The notes
//! are dynbrain's, word for word.

use serde::Serialize;

use super::rerank::ScoreSource;

pub const WEAK_MATCH_THRESHOLD: f64 = 0.70;

pub const NOTE_UNRATED: &str = "No confidence can be given for this search: the reranking model did not run, so these results are ordered by plain fusion of two searches and their scores are not comparable to the usual ones. THIS DOES NOT MEAN THE ANSWER IS MISSING -- it means the tool cannot judge. They are the same documents you would have got, only worse ordered: read them and decide for yourself.";

/// How sure a result list is that the corpus answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchConfidence {
    High,
    Low,
    Unrated,
}

/// The verdict on a result list, with the score of its best result.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConfidenceVerdict {
    pub confidence: MatchConfidence,
    pub top_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence_note: Option<String>,
}

pub fn verdict(top_score: Option<f64>, scoring: ScoreSource, threshold: f64) -> ConfidenceVerdict {
    let shown = top_score.map(|score| round_to(score, 4));
    let Some(top) = top_score.filter(|_| scoring == ScoreSource::CrossEncoder) else {
        return ConfidenceVerdict {
            confidence: MatchConfidence::Unrated,
            top_score: shown,
            confidence_note: Some(NOTE_UNRATED.to_owned()),
        };
    };
    if threshold > 0.0 && top < threshold {
        return ConfidenceVerdict {
            confidence: MatchConfidence::Low,
            top_score: shown,
            confidence_note: Some(low_note(top, threshold)),
        };
    }
    ConfidenceVerdict {
        confidence: MatchConfidence::High,
        top_score: shown,
        confidence_note: None,
    }
}

pub fn low_note(score: f64, threshold: f64) -> String {
    format!(
        "None of these results looks like an answer to your question: the best one scored {score:.2}, below the {threshold:.2} this documentation trusts. Most likely the answer is NOT documented. Say so, rather than assembling an answer out of these: use them only as related material, and tell the user you could not find it documented."
    )
}

pub fn round_to(value: f64, places: i32) -> f64 {
    let scale = 10f64.powi(places);
    (value * scale).round() / scale
}
