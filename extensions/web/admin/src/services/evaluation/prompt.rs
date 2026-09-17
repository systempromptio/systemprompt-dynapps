//! The prompts the judge receives, built the same way for the job and for
//! the dashboard page that shows them.

use crate::repositories::evaluation::inputs::AssembledInputs;
use crate::repositories::evaluation::{LoadedCriterion, Scoring};

pub const JUDGE_PREAMBLE: &str = "You are an evaluation judge for an organisation's AI-assisted work sessions. \
You will be shown labelled excerpts of one session and must return ONLY a JSON object matching the schema you were given. \
Be specific: every claim in the rationale must point at something in the excerpts, and every evidence item must quote it. \
Never invent content that is not in the excerpts. Write in plain English for an engineering manager.";

#[must_use]
pub fn build_system_prompt(criterion: &LoadedCriterion) -> String {
    let mode = match criterion.scoring {
        Scoring::Judge => {
            "Return a `score` from 0 to 100 following the rubric below, a `rationale`, and `evidence`."
        },
        Scoring::Metric => {
            "The score for this criterion is computed from the session's metrics and is given to you in the \
             user message. Do NOT return a score. Return a `rationale` that explains the given score from the \
             metrics and excerpts, and `evidence`."
        },
    };
    format!(
        "{JUDGE_PREAMBLE}\n\nCriterion: {}\n{mode}\n\n{}",
        criterion.name,
        criterion.instructions_markdown.trim()
    )
}

#[must_use]
pub fn build_user_prompt(inputs: &AssembledInputs, fixed_score: Option<u8>) -> String {
    let mut sections: Vec<String> = inputs
        .blocks
        .iter()
        .map(|block| format!("### {}\n{}", block.label, block.body))
        .collect();
    if let Some(score) = fixed_score {
        sections.push(format!(
            "### SCORE\nThe score is fixed at {score} by policy. Explain it; do not change it."
        ));
    }
    sections.join("\n\n")
}
