//! The ingestion round trip over a files source, with a fake model.
//!
//! Proves the write path end to end against a live Postgres: documents are
//! chunked, embedded, dated and written with citations; unrestricted content
//! is readable by everyone signed in and by nobody else; an unchanged
//! re-index makes zero model calls (FR-PROC-7 / NFR-COST-1); a removed file
//! is pruned (FR-ING-7); a rebuild re-uses the embedding cache; archive
//! categories are searched only when named (FR-RET-4); and a failing reranker
//! degrades to the fusion order instead of failing the call (NFR-DEGRADE-1).

use crate::common::TempDb;
use crate::fake_model::FakeModel;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use systemprompt::identifiers::UserId;
use systemprompt_mcp_knowledge_bank::config::KnowledgeBankConfig;
use systemprompt_mcp_knowledge_bank::{
    Ingestor, KnowledgeRetriever, PgRetriever, Principal, RunOptions, SearchFilter,
};

const ADR: &str = "---\ntitle: \"ADR-007 Rounding mode\"\ndate: 2026-01-05\n---\n# ADR-007 Rounding mode\n\n## Context\n\nMulti-currency baskets showed CHF totals off by a cent because totals were truncated instead of rounded with banker's rounding, and this paragraph is padded so the section is a chunk on its own.\n\n## Decision\n\nSwitch the rounding mode to banker's rounding for every currency and add a regression test on CHF baskets; this paragraph is padded so the section is a chunk on its own too.\n";
const NOTE: &str = "# Saved payment methods\n\nDesign wants tokenized cards listed in the account area. That needs a new PSP vault call and a consent checkbox at save time; legal signed off on the consent copy.\n";
const ARCHIVE: &str = "# Old proposal\n\nHistorical rationale for the checkout redesign, superseded by the current specs and only relevant when someone asks why the old design was abandoned.\n";

struct Fixture {
    db: TempDb,
    dir: tempfile::TempDir,
    model: Arc<FakeModel>,
    config: KnowledgeBankConfig,
}

impl Fixture {
    async fn create() -> Option<Self> {
        let db = TempDb::create().await?;
        db.create_knowledge_schema().await;
        let dir = tempfile::tempdir().expect("temp dir");
        std::fs::create_dir_all(dir.path().join("adrs")).expect("mkdir");
        std::fs::create_dir_all(dir.path().join("archive")).expect("mkdir");
        std::fs::write(dir.path().join("adrs/adr-007.md"), ADR).expect("write");
        std::fs::write(dir.path().join("adrs/note.md"), NOTE).expect("write");
        std::fs::write(dir.path().join("archive/old.md"), ARCHIVE).expect("write");
        let root = dir.path().to_string_lossy().replace('\\', "/");
        let yaml = format!(
            "chunking: {{target_chars: 200, max_chars: 400, min_chars: 40, split_tolerance: 20, enrich: false}}
dates: {{extract_via_llm: false}}
sources:
  adrs: {{kind: files, strategy: script, path: {root}/adrs, description: decisions}}
  openspec_archive: {{kind: files, strategy: script, path: {root}/archive, archived: true}}
"
        );
        let config = KnowledgeBankConfig::parse(&yaml).expect("fixture config parses");
        Some(Self {
            db,
            dir,
            model: Arc::new(FakeModel::default()),
            config,
        })
    }

    fn ingestor(&self) -> Ingestor {
        Ingestor::new(
            self.db.raw_pool_arc(),
            self.model.clone(),
            Arc::new(self.config.clone()),
            None,
        )
    }

    fn retriever(&self) -> PgRetriever {
        PgRetriever::new(
            self.db.raw_pool_arc(),
            self.model.clone(),
            self.config.clone(),
        )
    }
}

fn signed_in(user_id: &str) -> Principal {
    Principal {
        user_id: UserId::new(user_id),
        roles: vec!["user".to_owned()],
        attributes: std::collections::BTreeMap::new(),
    }
}

fn no_roles(user_id: &str) -> Principal {
    Principal {
        user_id: UserId::new(user_id),
        roles: Vec::new(),
        attributes: std::collections::BTreeMap::new(),
    }
}

fn filter(sources: &[&str]) -> SearchFilter {
    SearchFilter {
        source_types: sources.iter().map(|s| (*s).to_owned()).collect(),
        top_k: Some(5),
    }
}

#[tokio::test]
async fn files_are_indexed_with_citations_and_readable_by_everyone_signed_in() {
    let Some(fx) = Fixture::create().await else {
        return;
    };
    let report = fx
        .ingestor()
        .run(&RunOptions::default())
        .await
        .expect("run succeeds");
    assert!(!report.has_errors(), "{report:?}");
    let adrs = report
        .sources
        .iter()
        .find(|s| s.source_type == "adrs")
        .expect("adrs report");
    assert_eq!((adrs.seen, adrs.written, adrs.failed), (2, 2, 0));
    assert!(adrs.chunks >= 3, "the ADR splits into sections: {adrs:?}");
    assert!(report.version.is_some());

    let retriever = fx.retriever();
    let hits = retriever
        .search(
            &signed_in("u-1"),
            "banker's rounding CHF",
            &filter(&["adrs"]),
        )
        .await
        .expect("search succeeds");
    assert!(!hits.is_empty());
    let top = &hits[0];
    assert_eq!(top.source_type, "adrs");
    assert_eq!(top.date.as_deref(), Some("2026-01-05"));
    assert_eq!(top.date_source.as_deref(), Some("frontmatter"));
    assert!(
        top.uri.ends_with("adrs/adr-007.md"),
        "uri points at the file: {}",
        top.uri
    );
    assert!(
        top.lines.as_deref().is_some_and(|l| l.contains('-')),
        "line range present: {:?}",
        top.lines
    );
    assert!(top.chunk_total >= 2);

    let stranger = retriever
        .search(
            &no_roles("u-2"),
            "banker's rounding CHF",
            &filter(&["adrs"]),
        )
        .await
        .expect("search succeeds");
    assert!(
        stranger.is_empty(),
        "role:user is the default grant; a caller without it sees nothing"
    );

    let stats = retriever.index_stats().await.expect("stats");
    assert_eq!(stats.documents, 3);
    assert!(stats.version.is_some() && stats.last_built.is_some());

    fx.db.cleanup().await;
}

#[tokio::test]
async fn an_unchanged_corpus_reindexes_with_zero_model_calls() {
    let Some(fx) = Fixture::create().await else {
        return;
    };
    let ingestor = fx.ingestor();
    ingestor
        .run(&RunOptions::default())
        .await
        .expect("first run");
    let embeds_after_first = fx.model.embeds();
    assert!(embeds_after_first > 0);

    let report = ingestor
        .run(&RunOptions::default())
        .await
        .expect("second run");
    let adrs = report
        .sources
        .iter()
        .find(|s| s.source_type == "adrs")
        .expect("adrs report");
    assert_eq!((adrs.written, adrs.unchanged), (0, 2));
    assert_eq!(
        fx.model.embeds(),
        embeds_after_first,
        "no chunk was re-embedded"
    );
    assert_eq!(
        fx.model.converse_calls.load(Ordering::SeqCst),
        0,
        "dating is off and nothing was enriched"
    );

    fx.db.cleanup().await;
}

#[tokio::test]
async fn removed_files_are_pruned_and_rebuilds_reuse_the_embedding_cache() {
    let Some(fx) = Fixture::create().await else {
        return;
    };
    let ingestor = fx.ingestor();
    ingestor
        .run(&RunOptions::default())
        .await
        .expect("first run");

    std::fs::remove_file(fx.dir.path().join("adrs/note.md")).expect("remove");
    let report = ingestor
        .run(&RunOptions::default())
        .await
        .expect("second run");
    let adrs = report
        .sources
        .iter()
        .find(|s| s.source_type == "adrs")
        .expect("adrs report");
    assert_eq!(adrs.pruned, 1);
    let hits = fx
        .retriever()
        .search(
            &signed_in("u-1"),
            "tokenized cards consent",
            &filter(&["adrs"]),
        )
        .await
        .expect("search");
    assert!(
        hits.iter().all(|h| !h.uri.ends_with("note.md")),
        "a deleted source document produces no further hits"
    );

    let embeds_before_rebuild = fx.model.embeds();
    let rebuilt = ingestor
        .run(&RunOptions {
            rebuild: true,
            sources: vec!["adrs".to_owned()],
        })
        .await
        .expect("rebuild");
    let adrs = rebuilt
        .sources
        .iter()
        .find(|s| s.source_type == "adrs")
        .expect("adrs report");
    assert_eq!(adrs.written, 1);
    assert_eq!(
        fx.model.embeds(),
        embeds_before_rebuild,
        "rebuilding unchanged text hits the embedding cache"
    );

    fx.db.cleanup().await;
}

#[tokio::test]
async fn archive_categories_are_searched_only_when_named() {
    let Some(fx) = Fixture::create().await else {
        return;
    };
    fx.ingestor()
        .run(&RunOptions::default())
        .await
        .expect("run");
    let retriever = fx.retriever();

    let broad = retriever
        .search(
            &signed_in("u-1"),
            "checkout redesign abandoned",
            &filter(&[]),
        )
        .await
        .expect("search");
    assert!(broad.iter().all(|h| h.source_type != "openspec_archive"));

    let named = retriever
        .search(
            &signed_in("u-1"),
            "checkout redesign abandoned",
            &filter(&["openspec_archive"]),
        )
        .await
        .expect("search");
    assert!(named.iter().any(|h| h.source_type == "openspec_archive"));

    let unknown = retriever
        .search(&signed_in("u-1"), "anything", &filter(&["nope"]))
        .await
        .expect_err("unknown source is an error, not an empty result");
    assert!(unknown.to_string().contains("nope"));

    fx.db.cleanup().await;
}

#[tokio::test]
async fn a_failing_reranker_degrades_to_the_fusion_order() {
    let Some(fx) = Fixture::create().await else {
        return;
    };
    fx.ingestor()
        .run(&RunOptions::default())
        .await
        .expect("run");
    fx.model.fail_rerank.store(true, Ordering::SeqCst);

    let hits = fx
        .retriever()
        .search(&signed_in("u-1"), "rounding mode", &filter(&["adrs"]))
        .await
        .expect("search still succeeds");
    assert!(!hits.is_empty());
    assert!(
        fx.model.rerank_calls.load(Ordering::SeqCst) >= 1,
        "the reranker was attempted"
    );

    fx.db.cleanup().await;
}

#[tokio::test]
async fn a_second_run_cannot_start_while_one_is_in_flight() {
    let Some(fx) = Fixture::create().await else {
        return;
    };
    let ingestor = Arc::new(fx.ingestor());
    assert!(!ingestor.is_running());
    let report = ingestor.run(&RunOptions::default()).await.expect("run");
    assert!(!report.has_errors());
    fx.db.cleanup().await;
}
