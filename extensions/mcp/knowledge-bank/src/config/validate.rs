//! Load-time validation of the knowledge-bank config: every mistake that
//! would otherwise surface as a failed sync at 3am is a boot error instead.

use super::{KnowledgeBankConfig, SourceKind};
use crate::error::KnowledgeBankError;
use crate::principal::{ATTRIBUTE_SUBJECT_KINDS, SUBJECT_ROLE, SUBJECT_USER};
use std::str::FromStr;

// Why: derived from the principal's own vocabulary, never a second list — a
// grant kind the retrieval join can never match would be silently dead.
#[must_use]
pub fn grant_kinds() -> Vec<&'static str> {
    let mut kinds = vec![SUBJECT_USER, SUBJECT_ROLE];
    kinds.extend(ATTRIBUTE_SUBJECT_KINDS);
    kinds
}

pub(super) fn validate(config: &KnowledgeBankConfig) -> Result<(), KnowledgeBankError> {
    let mut problems = Vec::new();

    if !config.model.bedrock.region.starts_with("eu-") {
        problems.push(format!(
            "model.bedrock.region '{}' is outside the EU; indexed content is customer data",
            config.model.bedrock.region
        ));
    }
    if config.chunking.min_chars >= config.chunking.target_chars
        || config.chunking.target_chars > config.chunking.max_chars
    {
        problems.push("chunking must satisfy min_chars < target_chars <= max_chars".to_owned());
    }
    if config.retrieval.top_k == 0 || config.retrieval.candidate_multiplier == 0 {
        problems.push("retrieval.top_k and candidate_multiplier must be at least 1".to_owned());
    }
    if config.ingestion.doc_concurrency == 0 {
        problems.push("ingestion.doc_concurrency must be at least 1".to_owned());
    }
    if let Err(e) = croner::Cron::from_str(&config.ingestion.schedule) {
        problems.push(format!(
            "ingestion.schedule '{}' is not a valid cron expression: {e}",
            config.ingestion.schedule
        ));
    }

    for (id, source) in &config.sources {
        validate_source(id, source, &mut problems);
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(KnowledgeBankError::Config(problems.join("; ")))
    }
}

fn validate_source(id: &str, source: &super::SourceConfig, problems: &mut Vec<String>) {
    if !id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        problems.push(format!(
            "source '{id}': ids are lowercase snake_case (they become filter values)"
        ));
    }
    match source.kind {
        SourceKind::Files => {
            if source.path.as_deref().is_none_or(str::is_empty) {
                problems.push(format!("source '{id}': kind files needs a path"));
            }
        },
        SourceKind::Confluence => {
            if source.spaces.is_empty() {
                problems.push(format!("source '{id}': kind confluence needs spaces"));
            }
        },
        SourceKind::Jira => {
            if source.projects.is_empty() {
                problems.push(format!("source '{id}': kind jira needs projects"));
            }
        },
    }
    for pattern in source.include.iter().chain(&source.exclude) {
        if let Err(e) = glob::Pattern::new(pattern) {
            problems.push(format!("source '{id}': glob '{pattern}' is invalid: {e}"));
        }
    }
    let kinds = grant_kinds();
    for grant in &source.grants {
        if !kinds.contains(&grant.kind.as_str()) {
            problems.push(format!(
                "source '{id}': grant kind '{}' is not one of {}",
                grant.kind,
                kinds.join(", ")
            ));
        }
        if grant.value.trim().is_empty() {
            problems.push(format!("source '{id}': grant value must not be empty"));
        }
    }
}
