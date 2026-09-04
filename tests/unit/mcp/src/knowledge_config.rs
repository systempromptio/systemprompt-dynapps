//! The knowledge-bank configuration: the shipped file parses and validates,
//! every mistake that would otherwise surface as a failed sync is a load-time
//! error, and the catalogue carries files categories beside the Atlassian
//! ones with an archive opt-in. Adding a category is configuration, not code
//! (FR-ING-8); the model provider is configuration too (FR-MODEL-4).

use std::path::Path;
use systemprompt_mcp_knowledge_bank::config::{ProviderKind, SourceKind};
use systemprompt_mcp_knowledge_bank::error::KnowledgeBankError;
use systemprompt_mcp_knowledge_bank::{KnowledgeBankConfig, Subject};

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
fn the_shipped_config_carries_the_files_and_atlassian_categories() {
    let config = shipped();
    assert_eq!(config.model.provider, ProviderKind::Bedrock);
    assert!(config.model.bedrock.region.starts_with("eu-"));
    assert_eq!(config.sources["confluence"].kind, SourceKind::Confluence);
    assert_eq!(config.sources["jira"].kind, SourceKind::Jira);
    for id in ["meeting_notes", "adrs", "specs", "openspec_archive"] {
        assert_eq!(config.sources[id].kind, SourceKind::Files, "{id}");
    }
    assert!(config.sources["openspec_archive"].archived);
    assert_eq!(
        config.atlassian.base_url,
        "https://astounddigital.atlassian.net"
    );
    assert_eq!(config.knowledge_database.secret, "knowledge_database_url");
    assert_eq!(config.files.root, "storage/knowledge");
    assert_eq!(
        config.live_source_ids(),
        vec!["adrs", "confluence", "jira", "meeting_notes", "specs"]
    );
    assert_eq!(config.files_source_ids().len(), 4);
    assert!(config.requires_atlassian(&[]));
    assert!(!config.requires_atlassian(&["adrs".to_owned()]));
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
        "ingestion:\n  enabled: true\n  schedule: \"0 0 * * * *\"\n",
        "atlassian:\n  group_map: {}\n",
        "atlassian:\n  unmapped_groups: everyone\n",
        "bedrock:\n  region: eu-central-1\n",
    ] {
        assert!(
            KnowledgeBankConfig::parse(yaml).is_err(),
            "must be rejected: {yaml}"
        );
    }
}

#[test]
fn a_files_category_defaults_to_everyone_signed_in_under_the_storage_root() {
    let config = KnowledgeBankConfig::parse(
        "sources:\n  adrs:\n    kind: files\n    strategy: script\n  notes:\n    kind: files\n    strategy: llm\n    path: /srv/notes\n    grants: [{kind: adfs_group, value: Systemprompt-Core}]\n",
    )
    .expect("a files-only config needs no Atlassian site");
    let adrs = &config.sources["adrs"];
    assert_eq!(
        adrs.configured_subjects(),
        vec![Subject {
            kind: "role".to_owned(),
            value: "user".to_owned()
        }]
    );
    assert_eq!(
        adrs.resolved_path("adrs", &config.files),
        Path::new("storage/knowledge/adrs")
    );
    let notes = &config.sources["notes"];
    assert_eq!(
        notes.resolved_path("notes", &config.files),
        Path::new("/srv/notes")
    );
    assert_eq!(notes.configured_subjects()[0].kind, "adfs_group");
    assert!(!config.has_atlassian_sources());
    assert!(!config.requires_atlassian(&[]));
}

#[test]
fn the_files_root_can_be_overridden_by_the_deployment() {
    let config = shipped();
    assert_eq!(config.files.root, "storage/knowledge");
    let moved = config
        .clone()
        .with_files_root_override(Some(" /services/knowledge ".to_owned()));
    assert_eq!(moved.files.root, "/services/knowledge");
    assert_eq!(
        moved.sources["adrs"].resolved_path("adrs", &moved.files),
        Path::new("/services/knowledge/adrs")
    );
    let unchanged = config.with_files_root_override(Some("   ".to_owned()));
    assert_eq!(
        unchanged.files.root, "storage/knowledge",
        "blank means no override"
    );
}

#[test]
fn files_and_atlassian_fields_do_not_cross() {
    let message =
        error_of("sources:\n  adrs:\n    kind: files\n    strategy: script\n    spaces: [ENG]\n");
    assert!(message.contains("not spaces"), "{message}");
    let message = error_of(
        "atlassian: {base_url: https://x.atlassian.net}\nsources:\n  jira:\n    kind: jira\n    strategy: none\n    projects: [X]\n    grants: [{kind: role, value: user}]\n",
    );
    assert!(
        message.contains("grants apply to kind files only"),
        "{message}"
    );
    let message = error_of(
        "atlassian: {base_url: https://x.atlassian.net}\nsources:\n  confluence:\n    kind: confluence\n    strategy: script\n    spaces: [ENG]\n    include: ['**/*.md']\n",
    );
    assert!(message.contains("kind files only"), "{message}");
}

#[test]
fn a_bad_glob_or_grant_kind_fails_at_load() {
    let message = error_of(
        "sources:\n  adrs:\n    kind: files\n    strategy: script\n    include: ['[unclosed']\n",
    );
    assert!(message.contains("not a valid glob"), "{message}");
    let message = error_of(
        "sources:\n  adrs:\n    kind: files\n    strategy: script\n    grants: [{kind: organisation, value: x}]\n",
    );
    assert!(message.contains("grant kind 'organisation'"), "{message}");
    let message = error_of(
        "sources:\n  adrs:\n    kind: files\n    strategy: script\n    grants: [{kind: role, value: ''}]\n",
    );
    assert!(
        message.contains("grant value must not be empty"),
        "{message}"
    );
}

#[test]
fn an_unknown_model_provider_is_rejected_at_the_seam() {
    let parsed = KnowledgeBankConfig::parse("model:\n  provider: openai\n");
    let Err(KnowledgeBankError::Config(message)) = parsed else {
        panic!("an unsupported provider must be a config error, got {parsed:?}");
    };
    assert!(
        message.contains("openai") && message.contains("bedrock"),
        "the error names the unknown provider and the supported one: {message}"
    );
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

#[test]
fn the_server_is_gated_by_oauth_scope_and_a_deny_by_default_role_grant() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let server: serde_yaml::Value = serde_yaml::from_str(
        &std::fs::read_to_string(repo.join("services/mcp/knowledge-bank.yaml")).expect("mcp yaml"),
    )
    .expect("mcp yaml parses");
    let oauth = &server["mcp_servers"]["knowledge-bank"]["oauth"];
    assert_eq!(oauth["required"], serde_yaml::Value::Bool(true));
    assert_eq!(oauth["audience"].as_str(), Some("mcp"));
    assert_eq!(
        oauth["scopes"].as_sequence().map(Vec::len),
        Some(1),
        "one scope, `user`: an under-scoped token is refused by the transport"
    );
    assert_eq!(oauth["scopes"][0].as_str(), Some("user"));

    let roles: serde_yaml::Value = serde_yaml::from_str(
        &std::fs::read_to_string(repo.join("services/access-control/roles.yaml")).expect("roles"),
    )
    .expect("roles yaml parses");
    let rules = roles["rules"].as_sequence().expect("a rule list");
    let grant = rules.iter().find(|r| {
        r["entity_type"].as_str() == Some("mcp_server")
            && r["entity_id"].as_str() == Some("knowledge-bank")
    });
    let enabled =
        server["mcp_servers"]["knowledge-bank"]["enabled"] != serde_yaml::Value::Bool(false);
    if !enabled {
        assert!(
            grant.is_none(),
            "the server is disabled out of the box; its grant is withdrawn with it and returns \
             when `enabled` flips (a grant for an absent entity is dead ACL)"
        );
        return;
    }
    let grant = grant.expect("the knowledge-bank server has an explicit grant");
    assert_eq!(grant["access"].as_str(), Some("allow"));
    assert_eq!(
        grant["default_included"],
        serde_yaml::Value::Bool(false),
        "deny by default: a caller without a granted role never reaches the server (NFR-AUTHZ-1)"
    );
    let granted: Vec<&str> = grant["roles"]
        .as_sequence()
        .expect("roles")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(
        granted,
        ["user", "admin", "project_manager"],
        "every signed-in role may search; what each caller sees is decided per \
         document by knowledge_document_grants inside the retrieval queries"
    );
}
