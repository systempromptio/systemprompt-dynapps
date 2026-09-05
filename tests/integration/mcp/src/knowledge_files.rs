//! Files categories through the real ingestor against a live Postgres, with
//! a fake model: hand-authored notes, ADRs and spec trees are indexed from a
//! directory with citations and are readable by everyone signed in
//! (FR-ING-1, FR-ING-4); PDF and DOCX yield searchable text and an
//! unsupported file is skipped (FR-ING-5); a removed file leaves the index on
//! the next complete sync (FR-ING-7); a files sync needs no Atlassian
//! credentials at all; archive categories are opt-in (FR-RET-4); a changed
//! `grants:` list takes effect on rebuild; and a category added by
//! configuration alone is listed and searchable (FR-ING-8).

use crate::common::TempDb;
use crate::fake_model::FakeModel;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use systemprompt::identifiers::UserId;
use systemprompt_mcp_knowledge_bank::config::KnowledgeBankConfig;
use systemprompt_mcp_knowledge_bank::{
    FilesConnector, Ingestor, KnowledgeRetriever, PgRetriever, Principal, RunOptions, SearchFilter,
};

pub const ADR: &str = "---\ntitle: \"ADR-007 Rounding mode\"\ndate: 2026-01-05\n---\n# ADR-007 Rounding mode\n\n## Context\n\nMulti-currency baskets showed CHF totals off by a cent because totals were truncated instead of rounded with banker's rounding, and this paragraph is padded so the section is a chunk on its own.\n\n## Decision\n\nSwitch the rounding mode to banker's rounding for every currency and add a regression test on CHF baskets; this paragraph is padded so the section is a chunk on its own too.\n";
pub const NOTE: &str = "# Saved payment methods\n\nDesign wants tokenized cards listed in the account area. That needs a new PSP vault call and a consent checkbox at save time; legal signed off on the consent copy.\n";
pub const ARCHIVE: &str = "# Old proposal\n\nHistorical rationale for the checkout redesign, superseded by the current specs and only relevant when someone asks why the old design was abandoned.\n";

pub fn files_config(root: &Path, extra: &str) -> String {
    format!(
        "chunking: {{target_chars: 200, max_chars: 400, min_chars: 40, split_tolerance: 20, enrich: false}}
dates: {{extract_via_llm: false}}
files: {{root: {root}}}
atlassian: {{base_url: https://example.atlassian.net}}
sources:
  adrs: {{kind: files, strategy: script, description: decisions, include: ['**/*.md']}}
  meeting_notes: {{kind: files, strategy: script, description: notes}}
  openspec_archive: {{kind: files, strategy: script, archived: true, description: history}}
  confluence: {{kind: confluence, strategy: script, spaces: [ENG], description: pages}}
{extra}",
        root = root.display()
    )
}

pub struct Fixture {
    pub db: TempDb,
    pub model: Arc<FakeModel>,
    pub root: tempfile::TempDir,
    pub config: KnowledgeBankConfig,
}

impl Fixture {
    pub async fn create_or_skip() -> Option<Self> {
        Self::create_with_or_skip("").await
    }

    pub async fn create_with_or_skip(extra: &str) -> Option<Self> {
        let db = TempDb::create().await?;
        db.create_knowledge_schema().await;
        for user in ["u-alice", "u-bob"] {
            db.insert_user(user, &format!("{user}@example.invalid"))
                .await;
        }
        let root = tempfile::tempdir().expect("tempdir");
        write(root.path(), "adrs/007-rounding.md", ADR.as_bytes());
        write(
            root.path(),
            "meeting_notes/2026-01-20_payments.txt",
            NOTE.as_bytes(),
        );
        write(
            root.path(),
            "openspec_archive/old/proposal.md",
            ARCHIVE.as_bytes(),
        );
        let config = KnowledgeBankConfig::parse(&files_config(root.path(), extra))
            .expect("fixture config parses");
        Some(Self {
            db,
            model: Arc::new(FakeModel::default()),
            root,
            config,
        })
    }

    pub fn ingestor(&self) -> Ingestor {
        self.ingestor_with(self.config.clone())
    }

    pub fn ingestor_with(&self, config: KnowledgeBankConfig) -> Ingestor {
        Ingestor::new(self.db.raw_pool_arc(), self.model.clone(), Arc::new(config))
    }

    pub fn retriever(&self) -> PgRetriever {
        PgRetriever::new(
            self.db.raw_pool_arc(),
            self.model.clone(),
            self.config.clone(),
        )
    }

    pub async fn sync_files(&self, user: &str, options: &RunOptions) -> usize {
        let report = self
            .ingestor()
            .run_with(&signed_in(user), &FilesConnector, options)
            .await
            .expect("run succeeds");
        report
            .sources
            .iter()
            .filter(|s| s.source_type != "confluence")
            .for_each(|s| assert!(s.error.is_none(), "{s:?}"));
        report.sources.iter().map(|s| s.written).sum()
    }
}

pub fn write(root: &Path, rel: &str, bytes: &[u8]) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    std::fs::write(path, bytes).expect("write fixture");
}

pub fn signed_in(user_id: &str) -> Principal {
    Principal {
        user_id: UserId::new(user_id),
        roles: vec!["user".to_owned()],
        attributes: std::collections::BTreeMap::new(),
    }
}

pub fn no_roles(user_id: &str) -> Principal {
    Principal {
        user_id: UserId::new(user_id),
        roles: Vec::new(),
        attributes: std::collections::BTreeMap::new(),
    }
}

pub fn filter(sources: &[&str]) -> SearchFilter {
    SearchFilter {
        source_types: sources.iter().map(|s| (*s).to_owned()).collect(),
        top_k: Some(5),
    }
}

pub fn docx(paragraphs: &[&str]) -> Vec<u8> {
    let body: String = paragraphs
        .iter()
        .map(|p| format!("<w:p><w:r><w:t>{p}</w:t></w:r></w:p>"))
        .collect();
    let xml = format!("<w:document><w:body>{body}</w:body></w:document>");
    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut archive = zip::ZipWriter::new(&mut cursor);
        let options = zip::write::SimpleFileOptions::default();
        archive
            .start_file("word/document.xml", options)
            .expect("zip entry");
        archive.write_all(xml.as_bytes()).expect("zip body");
        archive.finish().expect("zip finish");
    }
    cursor.into_inner()
}

fn role_user() -> Vec<(String, String)> {
    vec![("role".to_owned(), "user".to_owned())]
}

#[tokio::test]
async fn files_are_indexed_with_citations_and_readable_by_everyone_signed_in() {
    let Some(fx) = Fixture::create_or_skip().await else {
        return;
    };
    assert_eq!(fx.sync_files("u-alice", &RunOptions::default()).await, 3);
    assert_eq!(
        fx.db.grants_of("adrs:007-rounding.md").await,
        role_user(),
        "a files document is granted to everyone signed in, not to the syncing caller"
    );

    let retriever = fx.retriever();
    let hits = retriever
        .search(
            &signed_in("u-bob"),
            "banker's rounding CHF",
            &filter(&["adrs"]),
        )
        .await
        .expect("search succeeds");
    let top = hits
        .first()
        .expect("bob never synced and still reads team ADRs");
    assert_eq!(top.source_type, "adrs");
    assert_eq!(top.source, "files");
    assert_eq!(top.date.as_deref(), Some("2026-01-05"));
    assert_eq!(top.date_source.as_deref(), Some("frontmatter"));
    assert!(top.uri.ends_with("/adrs/007-rounding.md"), "{}", top.uri);
    assert!(top.lines.as_deref().is_some_and(|l| l.contains('-')));

    let sources = retriever
        .list_sources(&signed_in("u-bob"))
        .await
        .expect("list");
    let adrs = sources
        .iter()
        .find(|s| s.id == "adrs")
        .expect("adrs listed");
    assert_eq!(adrs.doc_count, Some(1));
    let notes = sources
        .iter()
        .find(|s| s.id == "meeting_notes")
        .expect("notes listed");
    assert_eq!(notes.doc_count, Some(1));

    let nobody = retriever
        .search(
            &no_roles("u-guest"),
            "banker's rounding CHF",
            &filter(&["adrs"]),
        )
        .await
        .expect("search");
    assert!(nobody.is_empty(), "no role, no grant, no document");

    fx.db.cleanup().await;
}

#[tokio::test]
async fn a_pdf_and_a_docx_yield_searchable_text_and_an_unsupported_file_is_skipped() {
    let Some(fx) = Fixture::create_or_skip().await else {
        return;
    };
    let pdf = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/knowledge-transcript.pdf"),
    )
    .expect("pdf fixture");
    write(
        fx.root.path(),
        "meeting_notes/2026-02-01_transcript.pdf",
        &pdf,
    );
    write(
        fx.root.path(),
        "meeting_notes/2026-02-02_vault.docx",
        &docx(&[
            "Vault decision",
            "Tokenised cards need a consent checkbox and a PSP vault call.",
        ]),
    );
    write(fx.root.path(), "meeting_notes/whiteboard.png", b"\x89PNG");

    let report = fx
        .ingestor()
        .run_with(
            &signed_in("u-alice"),
            &FilesConnector,
            &RunOptions {
                sources: vec!["meeting_notes".to_owned()],
                ..RunOptions::default()
            },
        )
        .await
        .expect("run");
    let notes = &report.sources[0];
    assert_eq!(notes.written, 3, "{notes:?}");
    assert_eq!(
        notes.failed, 0,
        "a png is skipped, not counted as a failure"
    );

    let retriever = fx.retriever();
    let hits = retriever
        .search(
            &signed_in("u-bob"),
            "QA starts Tuesday CHF baskets",
            &filter(&["meeting_notes"]),
        )
        .await
        .expect("search");
    assert!(
        hits.iter()
            .any(|h| h.uri.ends_with("_transcript.pdf") && h.snippet.contains("QA starts Tuesday")),
        "the pdf text is searchable: {hits:?}"
    );
    let hits = retriever
        .search(
            &signed_in("u-bob"),
            "consent checkbox PSP vault",
            &filter(&["meeting_notes"]),
        )
        .await
        .expect("search");
    assert!(
        hits.iter().any(|h| h.uri.ends_with("_vault.docx")),
        "the docx text is searchable: {hits:?}"
    );

    fx.db.cleanup().await;
}

#[tokio::test]
async fn a_removed_file_leaves_the_index_on_the_next_complete_sync() {
    let Some(fx) = Fixture::create_or_skip().await else {
        return;
    };
    fx.sync_files("u-alice", &RunOptions::default()).await;
    assert_eq!(fx.db.count("knowledge_documents").await, 3);

    std::fs::remove_file(fx.root.path().join("meeting_notes/2026-01-20_payments.txt")).expect("rm");
    let report = fx
        .ingestor()
        .run_with(&signed_in("u-bob"), &FilesConnector, &RunOptions::default())
        .await
        .expect("run");
    let notes = report
        .sources
        .iter()
        .find(|s| s.source_type == "meeting_notes")
        .expect("notes report");
    assert_eq!(notes.pruned, 1);
    assert_eq!(fx.db.count("knowledge_documents").await, 2);
    assert_eq!(
        fx.db
            .grants_of("meeting_notes:2026-01-20_payments.txt")
            .await,
        Vec::<(String, String)>::new()
    );
    let hits = fx
        .retriever()
        .search(
            &signed_in("u-alice"),
            "tokenized cards consent",
            &filter(&["meeting_notes"]),
        )
        .await
        .expect("search");
    assert!(
        hits.is_empty(),
        "a deleted source document produces no further hits"
    );

    fx.db.cleanup().await;
}

#[tokio::test]
async fn a_files_sync_needs_no_atlassian_credentials() {
    let Some(fx) = Fixture::create_or_skip().await else {
        return;
    };
    let scoped = fx
        .ingestor()
        .run(
            &signed_in("u-alice"),
            None,
            &RunOptions {
                sources: vec!["adrs".to_owned()],
                ..RunOptions::default()
            },
        )
        .await
        .expect("a files-only run needs nothing from the caller");
    assert!(!scoped.has_errors(), "{scoped:?}");
    assert_eq!(scoped.sources[0].written, 1);

    let unscoped = fx
        .ingestor()
        .run(&signed_in("u-bob"), None, &RunOptions::default())
        .await
        .expect("the run itself succeeds");
    let confluence = unscoped
        .sources
        .iter()
        .find(|s| s.source_type == "confluence")
        .expect("confluence attempted");
    assert!(
        confluence
            .error
            .as_deref()
            .is_some_and(|e| e.contains("x-atlassian-api-token")),
        "the Atlassian category fails alone and names the header: {confluence:?}"
    );
    let written: usize = unscoped
        .sources
        .iter()
        .filter(|s| s.source_type != "confluence")
        .map(|s| s.written + s.unchanged)
        .sum();
    assert_eq!(written, 3, "every files category was still synced");

    fx.db.cleanup().await;
}

#[tokio::test]
async fn archive_file_categories_are_searched_only_when_named() {
    let Some(fx) = Fixture::create_or_skip().await else {
        return;
    };
    fx.sync_files("u-alice", &RunOptions::default()).await;
    let retriever = fx.retriever();
    let query = "checkout redesign abandoned";

    let broad = retriever
        .search(&signed_in("u-alice"), query, &filter(&[]))
        .await
        .expect("search");
    assert!(
        broad.iter().all(|h| h.source_type != "openspec_archive"),
        "an unscoped search means every LIVE category: {broad:?}"
    );
    let named = retriever
        .search(&signed_in("u-alice"), query, &filter(&["openspec_archive"]))
        .await
        .expect("search");
    assert_eq!(
        named.first().map(|h| h.source_type.as_str()),
        Some("openspec_archive")
    );

    fx.db.cleanup().await;
}

#[tokio::test]
async fn configured_grants_replace_the_default_on_rebuild_but_not_on_an_incremental_sync() {
    let Some(fx) = Fixture::create_or_skip().await else {
        return;
    };
    fx.sync_files("u-alice", &RunOptions::default()).await;
    assert_eq!(fx.db.grants_of("adrs:007-rounding.md").await, role_user());

    let narrowed = KnowledgeBankConfig::parse(&files_config(fx.root.path(), "").replace(
        "adrs: {kind: files, strategy: script, description: decisions, include: ['**/*.md']}",
        "adrs: {kind: files, strategy: script, description: decisions, include: ['**/*.md'], grants: [{kind: adfs_group, value: Systemprompt-Core}]}",
    ))
    .expect("narrowed config parses");
    let core_group = vec![("adfs_group".to_owned(), "Systemprompt-Core".to_owned())];

    fx.ingestor_with(narrowed.clone())
        .run_with(
            &signed_in("u-alice"),
            &FilesConnector,
            &RunOptions::default(),
        )
        .await
        .expect("incremental run");
    let mut both = role_user();
    both.extend(core_group.clone());
    both.sort();
    assert_eq!(
        fx.db.grants_of("adrs:007-rounding.md").await,
        both,
        "an incremental sync is additive"
    );

    fx.ingestor_with(narrowed)
        .run_with(
            &signed_in("u-alice"),
            &FilesConnector,
            &RunOptions {
                rebuild: true,
                ..RunOptions::default()
            },
        )
        .await
        .expect("rebuild");
    assert_eq!(
        fx.db.grants_of("adrs:007-rounding.md").await,
        core_group,
        "a rebuild replaces the configured grants"
    );
    let hits = fx
        .retriever()
        .search(
            &signed_in("u-bob"),
            "banker's rounding CHF",
            &filter(&["adrs"]),
        )
        .await
        .expect("search");
    assert!(hits.is_empty(), "bob is not in the group any more");

    fx.db.cleanup().await;
}

#[tokio::test]
async fn a_category_added_by_configuration_alone_is_listed_and_searchable() {
    let Some(fx) = Fixture::create_with_or_skip(
        "  runbooks: {kind: files, strategy: script, description: how we operate}\n",
    )
    .await
    else {
        return;
    };
    write(
        fx.root.path(),
        "runbooks/deploy.md",
        b"# Deploy runbook\n\nDeploy with just deploy-next and watch the health endpoint until it reports the new version.\n",
    );
    fx.sync_files("u-alice", &RunOptions::default()).await;

    let retriever = fx.retriever();
    let sources = retriever
        .list_sources(&signed_in("u-bob"))
        .await
        .expect("list");
    let runbooks = sources
        .iter()
        .find(|s| s.id == "runbooks")
        .expect("the new category is listed");
    assert_eq!(runbooks.doc_count, Some(1));
    assert_eq!(runbooks.description, "how we operate");
    let hits = retriever
        .search(
            &signed_in("u-bob"),
            "deploy-next health endpoint",
            &filter(&["runbooks"]),
        )
        .await
        .expect("search");
    assert_eq!(
        hits.first().map(|h| h.source_type.as_str()),
        Some("runbooks")
    );

    fx.db.cleanup().await;
}
