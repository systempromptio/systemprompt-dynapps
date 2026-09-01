//! The knowledge-bank configuration: the shipped file parses and validates,
//! every mistake that would otherwise surface as a failed sync is a load-time
//! error, and the catalogue is Atlassian-only with an archive opt-in.

use std::path::Path;
use systemprompt_mcp_knowledge_bank::KnowledgeBankConfig;
use systemprompt_mcp_knowledge_bank::config::{ProviderKind, SourceKind};
use systemprompt_mcp_knowledge_bank::error::KnowledgeBankError;

fn shipped() -> KnowledgeBankConfig {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../services/knowledge-bank/config.yaml");
    KnowledgeBankConfig::load(&path).expect("the shipped config loads")
}

fn error_of(yaml: &str) -> String {
    match KnowledgeBankConfig::parse(yaml) {
        Err(KnowledgeBankError::Config(message)) => message,
        Err(other) => panic!("expected a config error, got {other}"),
        Ok(_) => panic!("expected the config to be rejected"),
    }
}

#[test]
fn the_shipped_config_is_atlassian_only_on_the_astound_site() {
    let config = shipped();
    assert_eq!(config.model.provider, ProviderKind::Bedrock);
    assert!(config.model.bedrock.region.starts_with("eu-"));
    assert_eq!(config.sources.len(), 2, "confluence and jira, nothing else");
    assert_eq!(config.sources["confluence"].kind, SourceKind::Confluence);
    assert_eq!(config.sources["jira"].kind, SourceKind::Jira);
    assert_eq!(
        config.atlassian.base_url,
        "https://astounddigital.atlassian.net"
    );
    assert_eq!(config.knowledge_database.secret, "knowledge_database_url");
    assert_eq!(
        config.live_source_ids(),
        vec!["confluence".to_owned(), "jira".to_owned()]
    );
}

#[test]
fn a_non_eu_region_is_rejected_at_load() {
    let message = error_of("model:\n  bedrock:\n    region: us-east-1\n");
    assert!(message.contains("outside the EU"));
}

#[test]
fn atlassian_sources_need_scope_and_a_site() {
    let message = error_of(
        "atlassian: {base_url: https://x.atlassian.net}\nsources:\n  confluence:\n    kind: confluence\n    strategy: script\n",
    );
    assert!(message.contains("needs spaces"));
    let message = error_of(
        "atlassian: {base_url: https://x.atlassian.net}\nsources:\n  jira:\n    kind: jira\n    strategy: none\n",
    );
    assert!(message.contains("needs projects"));
    let message =
        error_of("sources:\n  jira:\n    kind: jira\n    strategy: none\n    projects: [X]\n");
    assert!(
        message.contains("atlassian.base_url is required"),
        "{message}"
    );
    let message = error_of(
        "atlassian: {base_url: http://x.atlassian.net}\nsources:\n  jira:\n    kind: jira\n    strategy: none\n    projects: [X]\n",
    );
    assert!(message.contains("https://"), "{message}");
}

#[test]
fn retired_keys_are_loud() {
    for yaml in [
        "sources:\n  adrs:\n    kind: files\n    strategy: script\n    path: x\n",
        "ingestion:\n  enabled: true\n  schedule: \"0 0 * * * *\"\n",
        "atlassian:\n  group_map: {}\n",
        "atlassian: {base_url: https://x.atlassian.net}\nsources:\n  jira:\n    kind: jira\n    strategy: none\n    projects: [X]\n    grants: [{kind: role, value: user}]\n",
        "bedrock:\n  region: eu-central-1\n",
    ] {
        assert!(
            KnowledgeBankConfig::parse(yaml).is_err(),
            "must be rejected: {yaml}"
        );
    }
}

#[test]
fn ids_and_concurrency_are_validated() {
    assert!(
        error_of(
            "atlassian: {base_url: https://x.atlassian.net}\nsources:\n  Meeting-Notes:\n    kind: jira\n    strategy: none\n    projects: [X]\n"
        )
        .contains("snake_case")
    );
    assert!(error_of("ingestion:\n  doc_concurrency: 0\n").contains("doc_concurrency"));
    assert!(
        error_of("knowledge_database:\n  secret: \"\"\n").contains("knowledge_database.secret")
    );
}
