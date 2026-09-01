//! Document dating with recorded provenance.
//!
//! Document dating: one effective date per document, resolved by priority —
//! a date the source system states (frontmatter), then the model's reading of
//! the opening text, then the file's modification time — with the provenance
//! recorded beside it, because the agent policy trusts them in that order and
//! an `mtime` date must never drive a supersede.

use crate::model::ModelProvider;
use chrono::{DateTime, NaiveDate, Utc};
use regex::Regex;
use std::sync::LazyLock;

pub const SOURCE_FRONTMATTER: &str = "frontmatter";
pub const SOURCE_LLM: &str = "llm";
pub const SOURCE_MTIME: &str = "mtime";
pub const SOURCE_NONE: &str = "none";

const LLM_INPUT_CHARS: usize = 2000;

const DATE_SYSTEM_PROMPT: &str = "You extract the single most representative date of a document: \
    the date it was created, or the date it describes — the meeting date for meeting notes, the \
    version date for a specification. Reply with only that date as YYYY-MM-DD. If there is no \
    clear date, reply with exactly the word NONE.";

static ISO_DATE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(\d{4}-\d{2}-\d{2})\b").unwrap_or_else(|e| unreachable!("date regex: {e}"))
});

#[must_use]
pub fn source_label(label: &str) -> &'static str {
    match label {
        SOURCE_FRONTMATTER => SOURCE_FRONTMATTER,
        SOURCE_LLM => SOURCE_LLM,
        SOURCE_MTIME => SOURCE_MTIME,
        _ => SOURCE_NONE,
    }
}

#[must_use]
pub fn frontmatter_date(body: &str) -> Option<NaiveDate> {
    let rest = body.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    let block = &rest[..end];
    for key in ["updated", "created", "date"] {
        let found = block
            .lines()
            .find_map(|line| line.strip_prefix(key).and_then(|r| r.strip_prefix(':')))
            .and_then(parse_iso_date);
        if found.is_some() {
            return found;
        }
    }
    None
}

// Why: the regex accepts any `\d{4}-\d{2}-\d{2}` shape; chrono then rejects
// impossible calendar dates so a hallucinated `2026-13-40` never gets cached.
#[must_use]
pub fn parse_iso_date(text: &str) -> Option<NaiveDate> {
    let captured = ISO_DATE.captures(text)?;
    NaiveDate::parse_from_str(captured.get(1)?.as_str(), "%Y-%m-%d").ok()
}

pub async fn llm_date(model: &dyn ModelProvider, body: &str) -> Option<NaiveDate> {
    let opening: String = body.chars().take(LLM_INPUT_CHARS).collect();
    match model.converse(DATE_SYSTEM_PROMPT, &opening).await {
        Ok(reply) => parse_iso_date(&reply),
        Err(e) => {
            tracing::warn!(error = %e, "llm dating failed; falling back to mtime");
            None
        },
    }
}

#[must_use]
pub fn mtime_date(mtime: Option<DateTime<Utc>>) -> Option<NaiveDate> {
    mtime.map(|t| t.date_naive())
}

/// Inputs to one resolution.
#[derive(Debug, Clone, Copy)]
pub struct DateInputs<'a> {
    pub body: &'a str,
    pub authored: Option<NaiveDate>,
    pub mtime: Option<DateTime<Utc>>,
    pub extract_via_llm: bool,
}

pub async fn resolve(
    inputs: DateInputs<'_>,
    model: Option<&dyn ModelProvider>,
) -> (Option<NaiveDate>, &'static str) {
    if let Some(date) = inputs.authored.or_else(|| frontmatter_date(inputs.body)) {
        return (Some(date), SOURCE_FRONTMATTER);
    }
    if inputs.extract_via_llm
        && let Some(model) = model
        && let Some(date) = llm_date(model, inputs.body).await
    {
        return (Some(date), SOURCE_LLM);
    }
    mtime_date(inputs.mtime).map_or((None, SOURCE_NONE), |date| (Some(date), SOURCE_MTIME))
}
