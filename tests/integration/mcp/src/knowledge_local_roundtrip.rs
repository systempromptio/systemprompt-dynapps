//! The shipped fixtures through the real pipeline with the local model
//! provider against a live pgvector Postgres — no credential, no network:
//! `project_docs`, `project_tickets` and `client_accounts` are ingested from
//! `services/knowledge-bank/fixtures/`, dated from their frontmatter, granted
//! to everyone signed in, and searchable by a user who never synced; a
//! no-role principal gets nothing; the sentinel phrase finds the Larkspur
//! brief; an uploaded note is found at once; and `index_stats` counts the
//! result. This is the path a fresh install takes with `just knowledge-seed`.

use crate::common::TempDb;
use crate::knowledge_files::{filter, no_roles, signed_in};
use std::sync::Arc;
use systemprompt_mcp_knowledge_bank::config::KnowledgeBankConfig;
use systemprompt_mcp_knowledge_bank::model::build_provider;
use systemprompt_mcp_knowledge_bank::{
    FilesConnector, Ingestor, KnowledgeRetriever, PgRetriever, RunOptions, UploadRequest,
};

const FIXTURE_CATEGORIES: [&str; 3] = ["project_docs", "project_tickets", "client_accounts"];

// Why: the shipped catalogue points the fixture categories at repo-relative
// paths; the test process runs from the crate directory, so they are
// re-anchored on the repository root. `scratch` is a temp category for the
// upload, which must never write into the tracked fixture tree.
fn config(scratch: &std::path::Path) -> KnowledgeBankConfig {
    let shipped = KnowledgeBankConfig::load(&astound_test_common::repo_path(
        "services/knowledge-bank/config.yaml",
    ))
    .expect("the shipped config loads");
    let mut config = KnowledgeBankConfig::parse(&format!(
        "model: {{provider: local}}
chunking: {{enrich: false}}
dates: {{extract_via_llm: false}}
files: {{root: {root}}}
sources:
  scratch: {{kind: files, strategy: script, description: scratch}}
",
        root = scratch.display()
    ))
    .expect("test config parses");
    for id in FIXTURE_CATEGORIES {
        let mut source = shipped.sources[id].clone();
        source.path = Some(
            astound_test_common::repo_path(&format!("services/knowledge-bank/fixtures/{id}"))
                .display()
                .to_string(),
        );
        config.sources.insert(id.to_owned(), source);
    }
    config
}

struct Harness {
    db: TempDb,
    ingestor: Ingestor,
    retriever: PgRetriever,
    _scratch: tempfile::TempDir,
}

impl Harness {
    async fn create_or_skip() -> Option<Self> {
        let db = TempDb::create().await?;
        db.create_knowledge_schema().await;
        for user in ["u-alice", "u-bob"] {
            db.insert_user(user, &format!("{user}@example.invalid"))
                .await;
        }
        let scratch = tempfile::tempdir().expect("tempdir");
        let config = config(scratch.path());
        let model = build_provider(&config.model, None).expect("the local provider builds");
        let ingestor = Ingestor::new(
            db.raw_pool_arc(),
            Arc::clone(&model),
            Arc::new(config.clone()),
        );
        let retriever = PgRetriever::new(db.raw_pool_arc(), model, config);
        Some(Self {
            db,
            ingestor,
            retriever,
            _scratch: scratch,
        })
    }
}

#[tokio::test]
async fn the_shipped_fixtures_index_with_the_local_provider_and_answer_for_everyone_signed_in() {
    let Some(h) = Harness::create_or_skip().await else {
        return;
    };
    let report = h
        .ingestor
        .run_with(
            &signed_in("u-alice"),
            &FilesConnector,
            &RunOptions {
                rebuild: false,
                sources: FIXTURE_CATEGORIES.iter().map(|s| (*s).to_owned()).collect(),
            },
        )
        .await
        .expect("sync succeeds");
    assert!(!report.has_errors(), "{report:?}");
    let written: std::collections::BTreeMap<&str, usize> = report
        .sources
        .iter()
        .map(|s| (s.source_type.as_str(), s.written))
        .collect();
    assert_eq!(written["project_docs"], 10);
    assert_eq!(written["project_tickets"], 20);
    assert_eq!(written["client_accounts"], 5);

    let bob = signed_in("u-bob");
    let hits = h
        .retriever
        .search(
            &bob,
            "express checkout feature flag",
            &filter(&["project_docs"]),
        )
        .await
        .expect("search");
    let top = hits
        .first()
        .expect("bob never synced and still reads the docs");
    assert_eq!(top.source_type, "project_docs");
    assert_eq!(top.date_source.as_deref(), Some("frontmatter"));
    assert!(
        top.uri.ends_with("checkout-express-fsd.md"),
        "the checkout FSD answers a checkout question: {}",
        top.uri
    );
    assert_eq!(top.date.as_deref(), Some("2026-07-14"));

    let hits = h
        .retriever
        .search(
            &bob,
            "The Larkspur pilot ships on the third Tuesday",
            &filter(&[]),
        )
        .await
        .expect("search");
    assert!(
        hits.first()
            .is_some_and(|hit| hit.uri.ends_with("larkspur-outdoors.md")),
        "the sentinel phrase finds the Larkspur brief: {:?}",
        hits.iter().map(|h| &h.uri).collect::<Vec<_>>()
    );

    let hits = h
        .retriever
        .search(
            &bob,
            "expired card shows as active",
            &filter(&["project_tickets"]),
        )
        .await
        .expect("search");
    assert!(
        hits.iter().any(|hit| hit.uri.ends_with("eaicomm-107.md")),
        "{:?}",
        hits.iter().map(|h| &h.uri).collect::<Vec<_>>()
    );

    let nobody = h
        .retriever
        .search(
            &no_roles("u-guest"),
            "express checkout feature flag",
            &filter(&[]),
        )
        .await
        .expect("search");
    assert!(nobody.is_empty(), "no role, no grant, no document");

    let sources = h.retriever.list_sources(&bob).await.expect("list");
    for (id, count) in [
        ("project_docs", 10),
        ("project_tickets", 20),
        ("client_accounts", 5),
    ] {
        let source = sources.iter().find(|s| s.id == id).expect(id);
        assert_eq!(source.doc_count, Some(count), "{id}");
        assert!(source.available, "{id}");
    }

    let upload = h
        .ingestor
        .add_document(
            &signed_in("u-alice"),
            UploadRequest {
                source_type: "scratch".to_owned(),
                file_name: "2026-09-05_standup.md".to_owned(),
                bytes: b"# Standup\n\nThe quokka migration finishes on Friday; Ben owns the quokka cutover checklist.\n".to_vec(),
                title: None,
                overwrite: false,
            },
        )
        .await
        .expect("upload succeeds");
    assert_eq!(upload.chunks, 1);
    let hits = h
        .retriever
        .search(&bob, "quokka cutover checklist", &filter(&["scratch"]))
        .await
        .expect("search");
    assert!(
        hits.first().is_some_and(|hit| hit.doc_id == upload.doc_id),
        "an uploaded note is searchable at once"
    );

    let stats = h.retriever.index_stats().await.expect("stats");
    assert_eq!(stats.documents, 36);
    assert!(stats.chunks >= stats.documents);
    assert_eq!(stats.grants, 36, "one role:user grant per document");
    assert_eq!(stats.embedding_cache_rows, stats.chunks);
    assert_eq!(h.db.count("knowledge_documents").await, 36);
    h.db.cleanup().await;
}
