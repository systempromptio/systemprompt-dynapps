//! The files connector and the credential rule of the shipped connector
//! (FR-ING-1, FR-ING-4, FR-ING-8): a directory tree per category is read
//! without any credential, unsupported files are skipped, globs select what
//! is taken, the archive flag is configuration, and a Confluence or Jira
//! category reached without a connected Atlassian account fails for that
//! source alone and names the fix.

use std::collections::HashSet;
use systemprompt_mcp_knowledge_bank::config::KnowledgeBankConfig;
use systemprompt_mcp_knowledge_bank::error::KnowledgeBankError;
use systemprompt_mcp_knowledge_bank::ingest::sources::FetchRequest;
use systemprompt_mcp_knowledge_bank::{ConfiguredConnector, FilesConnector, SourceConnector};

fn config(root: &std::path::Path) -> KnowledgeBankConfig {
    KnowledgeBankConfig::parse(&format!(
        "files: {{root: {}}}\natlassian: {{base_url: https://x.atlassian.net}}\nsources:\n  adrs: {{kind: files, strategy: script, include: ['**/*.md']}}\n  openspec_archive: {{kind: files, strategy: script, archived: true}}\n  confluence: {{kind: confluence, strategy: script, spaces: [ENG]}}\n",
        root.display()
    ))
    .expect("config parses")
}

fn request<'a>(
    config: &'a KnowledgeBankConfig,
    source_type: &'a str,
    known: &'a HashSet<String>,
) -> FetchRequest<'a> {
    FetchRequest {
        source_type,
        source: &config.sources[source_type],
        files: &config.files,
        last_sync: None,
        include_comments: true,
        known_doc_ids: known,
    }
}

#[tokio::test]
async fn a_files_category_is_read_from_its_directory_without_any_credential() {
    let dir = tempfile::tempdir().expect("tempdir");
    let adrs = dir.path().join("adrs");
    std::fs::create_dir_all(adrs.join("nested")).expect("mkdir");
    std::fs::write(
        adrs.join("001-rounding.md"),
        "# ADR-001 Rounding\n\nUse banker's rounding.",
    )
    .expect("write");
    std::fs::write(
        adrs.join("nested/002-vault.md"),
        "# ADR-002 Vault\n\nTokenise cards.",
    )
    .expect("write");
    std::fs::write(adrs.join("notes.txt"), "excluded by the include glob").expect("write");
    std::fs::write(adrs.join("diagram.png"), b"\x89PNG").expect("write");
    let config = config(dir.path());
    let known = HashSet::new();

    let outcome = ConfiguredConnector::new(None)
        .fetch(&request(&config, "adrs", &known))
        .await
        .expect("files fetch succeeds");

    assert!(outcome.complete, "a directory listing is always complete");
    let refs: Vec<&str> = outcome.current_refs.iter().map(String::as_str).collect();
    assert_eq!(refs, ["001-rounding.md", "nested/002-vault.md"]);
    let doc = &outcome.documents[0];
    assert_eq!(doc.doc_id, "adrs:001-rounding.md");
    assert_eq!(doc.source, "files");
    assert_eq!(doc.title, "ADR-001 Rounding");
    assert!(doc.uri.ends_with("/adrs/001-rounding.md"), "{}", doc.uri);
    assert!(
        doc.mtime.is_some(),
        "mtime dates a file when nothing else can"
    );
}

#[tokio::test]
async fn a_missing_directory_is_an_empty_complete_listing_not_an_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let config = config(dir.path());
    let known = HashSet::new();
    let outcome = FilesConnector
        .fetch(&request(&config, "openspec_archive", &known))
        .await
        .expect("an absent tree is empty");
    assert!(outcome.documents.is_empty() && outcome.complete);
    assert!(config.sources["openspec_archive"].archived);
}

#[tokio::test]
async fn an_unchanged_file_the_index_holds_is_skipped_on_an_incremental_run() {
    let dir = tempfile::tempdir().expect("tempdir");
    let adrs = dir.path().join("adrs");
    std::fs::create_dir_all(&adrs).expect("mkdir");
    std::fs::write(adrs.join("001.md"), "# One\n\nbody").expect("write");
    let config = config(dir.path());
    let known: HashSet<String> = ["adrs:001.md".to_owned()].into_iter().collect();
    let mut req = request(&config, "adrs", &known);
    req.last_sync = Some(chrono::Utc::now() + chrono::Duration::hours(1));

    let outcome = FilesConnector.fetch(&req).await.expect("fetch");
    assert_eq!(outcome.unchanged, vec!["adrs:001.md".to_owned()]);
    assert!(outcome.documents.is_empty());
    assert_eq!(outcome.current_refs, vec!["001.md".to_owned()]);
}

#[tokio::test]
async fn an_atlassian_category_without_a_connection_is_refused_with_the_fix() {
    let dir = tempfile::tempdir().expect("tempdir");
    let config = config(dir.path());
    let known = HashSet::new();
    let connector = ConfiguredConnector::new(None);
    let err = connector
        .fetch(&request(&config, "confluence", &known))
        .await
        .expect_err("confluence needs the caller's connected account");
    assert!(matches!(err, KnowledgeBankError::AtlassianUnavailable(_)));
    assert!(
        err.to_string().contains("connected-accounts"),
        "the refusal names where to connect: {err}"
    );
    let err = FilesConnector
        .fetch(&request(&config, "confluence", &known))
        .await
        .expect_err("the files connector never reads Atlassian");
    assert!(matches!(err, KnowledgeBankError::AtlassianUnavailable(_)));
}
