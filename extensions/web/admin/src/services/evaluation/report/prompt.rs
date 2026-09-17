//! The two prompts a report call is made of.
//!
//! The fixed preamble plus the operator's instructions as the system prompt,
//! and the digest as the user message, each block named so the model knows
//! what it is reading.

const PREAMBLE: &str = "You are writing the evaluation report for an AI gateway. \
Return ONLY a JSON object matching the response schema. The user message is a JSON \
digest of judge scores; every figure you cite must come from it and every id you \
reference (skill, conversation, session, version, category) must appear in it verbatim.";

const DIGEST_GUIDE: &str = "Blocks: `totals` — runs and mean scores for the window with \
the equal window before (`prior_*`); `skills` — per-skill means, worst quality first, with \
the window before (`quality_prior`); `versions` — per marketplace version means, \
`is_current` marks what is served now; `categories` — conversation categories with mean \
scores and average tokens and cost per conversation; `sessions` — the worst and best \
judged sessions with the judge's rationale and one quoted piece of evidence. \
`sessions_omitted` says how many outliers did not fit.";

#[must_use]
pub fn build_system_prompt(instructions_markdown: &str) -> String {
    format!(
        "{PREAMBLE}\n\n{DIGEST_GUIDE}\n\n{}",
        instructions_markdown.trim()
    )
}

#[must_use]
pub fn build_user_prompt(digest_json: &str) -> String {
    format!("### DIGEST\n{digest_json}")
}
