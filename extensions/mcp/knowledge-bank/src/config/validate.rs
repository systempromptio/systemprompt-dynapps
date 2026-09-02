//! Load-time validation of the knowledge-bank config: every mistake that
//! would otherwise surface as a failed sync is a boot error instead.

use super::{KnowledgeBankConfig, SourceConfig, SourceKind};
use crate::error::KnowledgeBankError;
use crate::principal::{ATTRIBUTE_SUBJECT_KINDS, SUBJECT_ROLE, SUBJECT_USER};

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
    if config.knowledge_database.secret.trim().is_empty() {
        problems.push("knowledge_database.secret must name a profile secret".to_owned());
    }
    if config.files.root.trim().is_empty() {
        problems.push("files.root must name a directory".to_owned());
    }
    if config.has_atlassian_sources() {
        let base_url = config.atlassian.base_url.trim();
        if base_url.is_empty() {
            problems.push(
                "atlassian.base_url is required when confluence or jira sources are configured (https://<site>.atlassian.net)"
                    .to_owned(),
            );
        } else if !base_url.starts_with("https://") {
            problems.push(format!(
                "atlassian.base_url '{base_url}' must start with https://"
            ));
        }
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

fn validate_source(id: &str, source: &SourceConfig, problems: &mut Vec<String>) {
    if !id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        problems.push(format!(
            "source '{id}': ids are lowercase snake_case (they become filter values)"
        ));
    }
    match source.kind {
        SourceKind::Files => validate_files_source(id, source, problems),
        SourceKind::Confluence => {
            if source.spaces.is_empty() {
                problems.push(format!("source '{id}': kind confluence needs spaces"));
            }
            reject_files_fields(id, source, problems);
        },
        SourceKind::Jira => {
            if source.projects.is_empty() {
                problems.push(format!("source '{id}': kind jira needs projects"));
            }
            reject_files_fields(id, source, problems);
        },
    }
}

fn validate_files_source(id: &str, source: &SourceConfig, problems: &mut Vec<String>) {
    if !source.spaces.is_empty() || !source.projects.is_empty() || source.jql.is_some() {
        problems.push(format!(
            "source '{id}': kind files takes path/include/exclude/grants, not spaces, projects or jql"
        ));
    }
    if source.path.as_deref().is_some_and(|p| p.trim().is_empty()) {
        problems.push(format!("source '{id}': path must not be empty"));
    }
    for pattern in source.include.iter().chain(&source.exclude) {
        if glob::Pattern::new(pattern).is_err() {
            problems.push(format!("source '{id}': '{pattern}' is not a valid glob"));
        }
    }
    for grant in &source.grants {
        let known = grant.kind == SUBJECT_USER
            || grant.kind == SUBJECT_ROLE
            || ATTRIBUTE_SUBJECT_KINDS.contains(&grant.kind.as_str());
        if !known {
            problems.push(format!(
                "source '{id}': grant kind '{}' is not one of user, role, {}",
                grant.kind,
                ATTRIBUTE_SUBJECT_KINDS.join(", ")
            ));
        }
        if grant.value.trim().is_empty() {
            problems.push(format!("source '{id}': grant value must not be empty"));
        }
    }
}

// Why: a configured grant on an Atlassian category would widen access past
// what Atlassian enforced for the syncing caller; refusing it at load is the
// only place that mistake is visible.
fn reject_files_fields(id: &str, source: &SourceConfig, problems: &mut Vec<String>) {
    if source.path.is_some() || !source.include.is_empty() || !source.exclude.is_empty() {
        problems.push(format!(
            "source '{id}': path, include and exclude apply to kind files only"
        ));
    }
    if !source.grants.is_empty() {
        problems.push(format!(
            "source '{id}': grants apply to kind files only; Atlassian documents are granted to whoever synced them"
        ));
    }
}
