//! Load-time validation of the knowledge-bank config: every mistake that
//! would otherwise surface as a failed sync is a boot error instead.

use super::KnowledgeBankConfig;
use crate::error::KnowledgeBankError;

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
    if !config.sources.is_empty() {
        let base_url = config.atlassian.base_url.trim();
        if base_url.is_empty() {
            problems.push(
                "atlassian.base_url is required when sources are configured (https://<site>.atlassian.net)"
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
        super::SourceKind::Confluence => {
            if source.spaces.is_empty() {
                problems.push(format!("source '{id}': kind confluence needs spaces"));
            }
        },
        super::SourceKind::Jira => {
            if source.projects.is_empty() {
                problems.push(format!("source '{id}': kind jira needs projects"));
            }
        },
    }
}
