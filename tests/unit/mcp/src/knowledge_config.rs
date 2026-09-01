//! The knowledge-bank configuration: the shipped file parses and validates,
//! every mistake that would otherwise surface as a failed sync is a load-time
//! error, and the source catalogue exposes archive opt-in and default grants
//! the way retrieval and ingestion rely on.

use std::path::Path;
use systemprompt_mcp_knowledge_bank::KnowledgeBankConfig;
use systemprompt_mcp_knowledge_bank::config::{ProviderKind, SourceKind, grant_kinds};
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
fn the_shipped_config_names_every_phase0_source_with_an_archive_opt_in() {
    let config = shipped();
    assert_eq!(config.model.provider, ProviderKind::Bedrock);
    assert!(config.model.bedrock.region.starts_with("eu-"));
    for id in [
        "confluence",
        "jira",
        "meeting_notes",
        "adrs",
        "specs",
        "openspec_archive",
    ] {
        assert!(config.sources.contains_key(id), "missing source {id}");
    }
    let live = config.live_source_ids();
    assert!(!live.contains(&"openspec_archive".to_owned()));
    assert!(live.contains(&"confluence".to_owned()));
    assert_eq!(config.sources["confluence"].kind, SourceKind::Confluence);
    assert_eq!(config.sources["jira"].kind, SourceKind::Jira);
    assert_eq!(config.sources["adrs"].kind, SourceKind::Files);
    assert!(
        !config.ingestion.enabled,
        "ingestion is opt-in per deployment"
    );
}

#[test]
fn omitted_grants_default_to_everyone_signed_in() {
    let config = shipped();
    let defaults = config.sources["adrs"].default_grants();
    assert_eq!(defaults.len(), 1);
    assert_eq!(
        (defaults[0].kind.as_str(), defaults[0].value.as_str()),
        ("role", "user")
    );
}

#[test]
fn a_non_eu_region_is_rejected_at_load() {
    let message = error_of("model:\n  bedrock:\n    region: us-east-1\n");
    assert!(message.contains("outside the EU"));
}

#[test]
fn a_files_source_needs_a_path_and_atlassian_sources_need_scope() {
    let message = error_of("sources:\n  adrs:\n    kind: files\n    strategy: script\n");
    assert!(message.contains("needs a path"));
    let message = error_of("sources:\n  confluence:\n    kind: confluence\n    strategy: script\n");
    assert!(message.contains("needs spaces"));
    let message = error_of("sources:\n  jira:\n    kind: jira\n    strategy: none\n");
    assert!(message.contains("needs projects"));
}

#[test]
fn grant_kinds_are_limited_to_the_authz_vocabulary() {
    let message = error_of(
        "sources:\n  adrs:\n    kind: files\n    strategy: script\n    path: x\n    grants:\n      - kind: team\n        value: nava\n",
    );
    assert!(message.contains("grant kind 'team'"));
    for kind in grant_kinds() {
        assert!(message.contains(kind));
    }
}

#[test]
fn bad_schedules_globs_and_ids_are_rejected() {
    assert!(error_of("ingestion:\n  schedule: not a cron\n").contains("cron"));
    assert!(
        error_of("sources:\n  adrs:\n    kind: files\n    strategy: script\n    path: x\n    include: [\"[\"]\n")
            .contains("glob")
    );
    assert!(
        error_of(
            "sources:\n  Meeting-Notes:\n    kind: files\n    strategy: script\n    path: x\n"
        )
        .contains("snake_case")
    );
}

#[test]
fn unknown_keys_are_loud() {
    assert!(
        KnowledgeBankConfig::parse("bedrock:\n  region: eu-central-1\n").is_err(),
        "the old top-level bedrock block moved under model"
    );
}
