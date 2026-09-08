//! The shared-index write path, through an in-memory connector and a fake
//! model, against a live Postgres.
//!
//! What the remote, per-user model promises: a user's sync grants what they
//! fetched to them alone; a second user syncing the same page adds a grant,
//! not a document or an embedding; a page that disappears for one user loses
//! only that user's grant while another still holds it; a page nobody holds
//! is deleted; `prune_knowledge` removes a stale or vanished user's grants and
//! the documents and cache rows left behind; an unchanged corpus re-syncs
//! with zero model calls; incremental runs skip only what the index already
//! holds; archive categories are searched only when named; and a failing
//! reranker degrades to the fusion order (NFR-DEGRADE-1).

use crate::common::TempDb;
use crate::fake_model::FakeModel;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, HashSet};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use systemprompt::identifiers::UserId;
use systemprompt_mcp_knowledge_bank::config::KnowledgeBankConfig;
use systemprompt_mcp_knowledge_bank::error::KnowledgeBankError;
use systemprompt_mcp_knowledge_bank::ingest::document::{
    FetchOutcome, SourceDocument, content_hash,
};
use systemprompt_mcp_knowledge_bank::ingest::sources::{FetchRequest, SourceConnector};
use systemprompt_mcp_knowledge_bank::{
    Ingestor, KnowledgeRetriever, PgRetriever, Principal, PruneOptions, Pruner, RunOptions,
    SearchFilter,
};

const ADR: &str = "---\ntitle: \"ADR-007 Rounding mode\"\ndate: 2026-01-05\n---\n# ADR-007 Rounding mode\n\n## Context\n\nMulti-currency baskets showed CHF totals off by a cent because totals were truncated instead of rounded with banker's rounding, and this paragraph is padded so the section is a chunk on its own.\n\n## Decision\n\nSwitch the rounding mode to banker's rounding for every currency and add a regression test on CHF baskets; this paragraph is padded so the section is a chunk on its own too.\n";
const NOTE: &str = "# Saved payment methods\n\nDesign wants tokenized cards listed in the account area. That needs a new PSP vault call and a consent checkbox at save time; legal signed off on the consent copy.\n";
const ARCHIVE: &str = "# Old proposal\n\nHistorical rationale for the checkout redesign, superseded by the current specs and only relevant when someone asks why the old design was abandoned.\n";

const CONFIG: &str = "chunking: {target_chars: 200, max_chars: 400, min_chars: 40, split_tolerance: 20, enrich: false}
dates: {extract_via_llm: false}
atlassian: {base_url: https://example.atlassian.net}
sources:
  confluence: {kind: confluence, strategy: script, spaces: [ENG], description: pages}
  openspec_archive: {kind: confluence, strategy: script, spaces: [OLD], archived: true}
";

// What a user's Atlassian account can see, per source: (page id, title, body).
type Corpus = BTreeMap<String, Vec<(String, String, String)>>;

// An in-memory Confluence: every page has the same `updated` stamp, and a
// listing is always complete, so grant removal is exercised on every run.
struct FakeConnector {
    corpus: Mutex<Corpus>,
    updated: DateTime<Utc>,
}

impl FakeConnector {
    fn new(pages: &[(&str, &str, &str, &str)]) -> Self {
        let mut corpus = Corpus::new();
        for (source, id, title, body) in pages {
            corpus.entry((*source).to_owned()).or_default().push((
                (*id).to_owned(),
                (*title).to_owned(),
                (*body).to_owned(),
            ));
        }
        Self {
            corpus: Mutex::new(corpus),
            updated: Utc::now(),
        }
    }

    fn remove(&self, source: &str, id: &str) {
        let mut corpus = self.corpus.lock().expect("corpus lock");
        if let Some(pages) = corpus.get_mut(source) {
            pages.retain(|(page, _, _)| page != id);
        }
    }
}

#[async_trait]
impl SourceConnector for FakeConnector {
    async fn fetch(&self, request: &FetchRequest<'_>) -> Result<FetchOutcome, KnowledgeBankError> {
        let pages = self
            .corpus
            .lock()
            .expect("corpus lock")
            .get(request.source_type)
            .cloned()
            .unwrap_or_default();
        let mut outcome = FetchOutcome {
            complete: true,
            ..FetchOutcome::default()
        };
        for (id, title, body) in pages {
            outcome.current_refs.push(id.clone());
            let doc_id = SourceDocument::doc_id_for(request.source_type, &id);
            if request.can_skip(&doc_id, Some(self.updated)) {
                outcome.unchanged.push(doc_id);
                continue;
            }
            outcome.documents.push(SourceDocument {
                doc_id,
                source_type: request.source_type.to_owned(),
                source: "confluence".to_owned(),
                source_ref: id.clone(),
                title,
                uri: format!("https://example.atlassian.net/wiki/pages/{id}"),
                content_hash: content_hash(&body),
                body,
                mtime: Some(self.updated),
                authored_date: Some(self.updated.date_naive()),
            });
        }
        Ok(outcome)
    }
}

struct Fixture {
    db: TempDb,
    model: Arc<FakeModel>,
    config: KnowledgeBankConfig,
    connector: FakeConnector,
}

impl Fixture {
    async fn create_or_skip() -> Option<Self> {
        let db = TempDb::create().await?;
        db.create_knowledge_schema().await;
        for user in ["u-alice", "u-bob"] {
            db.insert_user(user, &format!("{user}@example.invalid"))
                .await;
        }
        Some(Self {
            db,
            model: Arc::new(FakeModel::default()),
            config: KnowledgeBankConfig::parse(CONFIG).expect("fixture config parses"),
            connector: FakeConnector::new(&[
                ("confluence", "adr-007", "ADR-007 Rounding mode", ADR),
                ("confluence", "note", "Saved payment methods", NOTE),
                ("openspec_archive", "old", "Old proposal", ARCHIVE),
            ]),
        })
    }

    fn ingestor(&self) -> Ingestor {
        Ingestor::new(
            self.db.raw_pool_arc(),
            self.model.clone(),
            Arc::new(self.config.clone()),
        )
    }

    fn retriever(&self) -> PgRetriever {
        PgRetriever::new(
            self.db.raw_pool_arc(),
            self.model.clone(),
            self.config.clone(),
        )
    }

    fn pruner(&self) -> Pruner {
        Pruner::new(self.db.raw_pool_arc(), self.db.raw_pool_arc())
    }

    async fn sync_as(&self, user: &str, options: &RunOptions) -> usize {
        let report = self
            .ingestor()
            .run_with(&signed_in(user), &self.connector, options)
            .await
            .expect("run succeeds");
        assert!(!report.has_errors(), "{report:?}");
        report.sources.iter().map(|s| s.written).sum()
    }
}

fn signed_in(user_id: &str) -> Principal {
    Principal {
        user_id: UserId::new(user_id),
        roles: vec!["user".to_owned()],
        attributes: std::collections::BTreeMap::new(),
    }
}

fn filter(sources: &[&str]) -> SearchFilter {
    SearchFilter {
        project_ids: Vec::new(),
        exclude_shared: false,
        source_types: sources.iter().map(|s| (*s).to_owned()).collect(),
        top_k: Some(5),
    }
}

fn user_grants(users: &[&str]) -> Vec<(String, String)> {
    let mut grants: Vec<(String, String)> = users
        .iter()
        .map(|u| ("user".to_owned(), (*u).to_owned()))
        .collect();
    grants.sort();
    grants
}

#[tokio::test]
async fn a_sync_indexes_with_citations_and_grants_only_the_caller() {
    let Some(fx) = Fixture::create_or_skip().await else {
        return;
    };
    assert_eq!(fx.sync_as("u-alice", &RunOptions::default()).await, 3);
    assert_eq!(
        fx.db.grants_of("confluence:adr-007").await,
        user_grants(&["u-alice"])
    );

    let retriever = fx.retriever();
    let hits = retriever
        .search(
            &signed_in("u-alice"),
            "banker's rounding CHF",
            &filter(&["confluence"]),
        )
        .await
        .expect("search succeeds");
    let top = hits.first().expect("alice sees her page");
    assert_eq!(top.source_type, "confluence");
    assert_eq!(
        top.date.as_deref(),
        Some(fx.connector.updated.format("%Y-%m-%d").to_string().as_str()),
        "the page version date outranks a date written into the body"
    );
    assert_eq!(top.date_source.as_deref(), Some("frontmatter"));
    assert!(top.uri.ends_with("/wiki/pages/adr-007"), "{}", top.uri);
    assert!(top.lines.as_deref().is_some_and(|l| l.contains('-')));
    assert!(top.chunk_total >= 2);

    let bob = retriever
        .search(
            &signed_in("u-bob"),
            "banker's rounding CHF",
            &filter(&["confluence"]),
        )
        .await
        .expect("search succeeds");
    assert!(
        bob.is_empty(),
        "a user who has not synced holds no grant and sees nothing, whatever their role"
    );

    let sources = retriever
        .list_sources(&signed_in("u-alice"))
        .await
        .expect("listing");
    let confluence = sources
        .iter()
        .find(|s| s.id == "confluence")
        .expect("confluence listed");
    assert_eq!(confluence.doc_count, Some(2));
    assert!(confluence.last_synced.is_some(), "alice's own last sync");
    let bob_sources = retriever
        .list_sources(&signed_in("u-bob"))
        .await
        .expect("listing");
    assert!(
        bob_sources.iter().all(|s| s.last_synced.is_none()),
        "bob has never synced"
    );

    let stats = retriever.index_stats().await.expect("stats");
    assert_eq!(
        (stats.documents, stats.grants, stats.syncing_users),
        (3, 3, 1)
    );
    assert!(stats.database_bytes > 0 && stats.embedding_cache_rows > 0);

    fx.db.cleanup().await;
}

#[tokio::test]
async fn a_second_reader_adds_a_grant_not_a_document_or_an_embedding() {
    let Some(fx) = Fixture::create_or_skip().await else {
        return;
    };
    fx.sync_as("u-alice", &RunOptions::default()).await;
    let embeds = fx.model.embeds();
    let cache_rows = fx.db.count("knowledge_embedding_cache").await;

    let written = fx.sync_as("u-bob", &RunOptions::default()).await;
    assert_eq!(written, 0, "bob's pages were already indexed by alice");
    assert_eq!(fx.db.count("knowledge_documents").await, 3);
    assert_eq!(
        fx.db.grants_of("confluence:adr-007").await,
        user_grants(&["u-alice", "u-bob"])
    );
    assert_eq!(fx.model.embeds(), embeds, "nothing was re-embedded");
    assert_eq!(fx.db.count("knowledge_embedding_cache").await, cache_rows);

    let hits = fx
        .retriever()
        .search(&signed_in("u-bob"), "banker's rounding CHF", &filter(&[]))
        .await
        .expect("search");
    assert!(!hits.is_empty(), "bob now reads the shared document");

    fx.db.cleanup().await;
}

#[tokio::test]
async fn a_page_that_disappears_loses_one_readers_grant_and_dies_with_the_last() {
    let Some(fx) = Fixture::create_or_skip().await else {
        return;
    };
    fx.sync_as("u-alice", &RunOptions::default()).await;
    fx.sync_as("u-bob", &RunOptions::default()).await;

    fx.connector.remove("confluence", "note");
    let report = fx
        .ingestor()
        .run_with(&signed_in("u-alice"), &fx.connector, &RunOptions::default())
        .await
        .expect("alice re-syncs");
    let confluence = report
        .sources
        .iter()
        .find(|s| s.source_type == "confluence")
        .expect("report");
    assert_eq!(confluence.pruned, 1, "alice's grant on the vanished page");
    assert_eq!(
        fx.db.grants_of("confluence:note").await,
        user_grants(&["u-bob"]),
        "bob still holds the page, so it stays"
    );
    assert_eq!(fx.db.count("knowledge_documents").await, 3);

    fx.sync_as("u-bob", &RunOptions::default()).await;
    assert!(fx.db.grants_of("confluence:note").await.is_empty());
    assert_eq!(
        fx.db.count("knowledge_documents").await,
        2,
        "nobody holds it any more, so it is deleted"
    );
    let hits = fx
        .retriever()
        .search(&signed_in("u-bob"), "tokenized cards consent", &filter(&[]))
        .await
        .expect("search");
    assert!(hits.iter().all(|h| !h.uri.ends_with("/note")));

    fx.db.cleanup().await;
}

#[tokio::test]
async fn prune_removes_stale_and_vanished_users_and_what_only_they_held() {
    let Some(fx) = Fixture::create_or_skip().await else {
        return;
    };
    fx.db.insert_user("u-gone", "gone@example.invalid").await;
    fx.sync_as("u-alice", &RunOptions::default()).await;
    fx.sync_as("u-gone", &RunOptions::default()).await;
    fx.connector.remove("confluence", "note");
    fx.sync_as("u-alice", &RunOptions::default()).await;
    assert_eq!(
        fx.db.grants_of("confluence:note").await,
        user_grants(&["u-gone"]),
        "only the departed user still holds the note"
    );
    sqlx::query("DELETE FROM users WHERE id = 'u-gone'")
        .execute(&*fx.db.raw_pool_arc())
        .await
        .expect("delete user");
    fx.db.age_sync_state("u-alice", 100).await;

    let pruner = fx.pruner();
    let dry = pruner
        .prune(PruneOptions {
            stale_days: 90,
            dry_run: true,
        })
        .await
        .expect("dry run");
    assert!(dry.dry_run);
    assert_eq!(dry.missing_users, vec!["u-gone".to_owned()]);
    assert_eq!(dry.stale_users, vec!["u-alice".to_owned()]);
    assert_eq!(dry.documents_deleted, 3, "both users gone: everything");
    assert_eq!(
        fx.db.count("knowledge_documents").await,
        3,
        "a dry run changes nothing"
    );

    let applied = pruner
        .prune(PruneOptions {
            stale_days: 365,
            dry_run: false,
        })
        .await
        .expect("prune");
    assert!(applied.stale_users.is_empty(), "alice is fresh at 365 days");
    assert_eq!(applied.missing_users, vec!["u-gone".to_owned()]);
    assert_eq!(applied.grants_removed, 3);
    assert_eq!(applied.sync_states_removed, 2, "one row per source");
    assert_eq!(applied.documents_deleted, 1, "the note only u-gone held");
    assert_eq!(applied.date_cache_removed, 1);
    assert_eq!(fx.db.count("knowledge_documents").await, 2);
    assert_eq!(
        fx.db.grants_of("confluence:adr-007").await,
        user_grants(&["u-alice"])
    );

    fx.db.cleanup().await;
}

#[tokio::test]
async fn an_unchanged_corpus_resyncs_with_zero_model_calls_and_rebuild_reuses_the_cache() {
    let Some(fx) = Fixture::create_or_skip().await else {
        return;
    };
    let ingestor = fx.ingestor();
    let alice = signed_in("u-alice");
    ingestor
        .run_with(&alice, &fx.connector, &RunOptions::default())
        .await
        .expect("first run");
    let embeds = fx.model.embeds();
    assert!(embeds > 0);

    let report = ingestor
        .run_with(&alice, &fx.connector, &RunOptions::default())
        .await
        .expect("second run");
    let confluence = report
        .sources
        .iter()
        .find(|s| s.source_type == "confluence")
        .expect("report");
    assert_eq!((confluence.written, confluence.unchanged), (0, 2));
    assert_eq!(fx.model.embeds(), embeds, "no chunk was re-embedded");
    assert_eq!(fx.model.converse_calls.load(Ordering::SeqCst), 0);

    let rebuilt = ingestor
        .run_with(
            &alice,
            &fx.connector,
            &RunOptions {
                rebuild: true,
                sources: vec!["confluence".to_owned()],
            },
        )
        .await
        .expect("rebuild");
    let confluence = rebuilt
        .sources
        .iter()
        .find(|s| s.source_type == "confluence")
        .expect("report");
    assert_eq!(
        confluence.written, 2,
        "a rebuild re-chunks everything fetched"
    );
    assert_eq!(
        fx.model.embeds(),
        embeds,
        "rebuilding unchanged text hits the embedding cache"
    );

    fx.db.cleanup().await;
}

#[tokio::test]
async fn an_incremental_skip_needs_the_index_to_hold_the_document() {
    let known: HashSet<String> = ["confluence:present".to_owned()].into_iter().collect();
    let config = KnowledgeBankConfig::parse(CONFIG).expect("config");
    let now = Utc::now();
    let request = FetchRequest {
        source_type: "confluence",
        source: &config.sources["confluence"],
        files: &config.files,
        last_sync: Some(now),
        include_comments: true,
        known_doc_ids: &known,
    };
    let older = Some(now - chrono::Duration::hours(1));
    assert!(request.can_skip("confluence:present", older));
    assert!(
        !request.can_skip("confluence:orphaned", older),
        "deleted as an orphan since the cursor: must be read again"
    );
    assert!(!request.can_skip("confluence:present", Some(now + chrono::Duration::hours(1))));
    assert!(!request.can_skip("confluence:present", None));
}

#[tokio::test]
async fn archive_categories_are_searched_only_when_named() {
    let Some(fx) = Fixture::create_or_skip().await else {
        return;
    };
    fx.sync_as("u-alice", &RunOptions::default()).await;
    let retriever = fx.retriever();

    let broad = retriever
        .search(
            &signed_in("u-alice"),
            "checkout redesign abandoned",
            &filter(&[]),
        )
        .await
        .expect("search");
    assert!(broad.iter().all(|h| h.source_type != "openspec_archive"));

    let named = retriever
        .search(
            &signed_in("u-alice"),
            "checkout redesign abandoned",
            &filter(&["openspec_archive"]),
        )
        .await
        .expect("search");
    assert!(named.iter().any(|h| h.source_type == "openspec_archive"));

    let unknown = retriever
        .search(&signed_in("u-alice"), "anything", &filter(&["nope"]))
        .await
        .expect_err("unknown source is an error, not an empty result");
    assert!(unknown.to_string().contains("nope"));

    fx.db.cleanup().await;
}

#[tokio::test]
async fn a_failing_reranker_degrades_to_the_fusion_order() {
    let Some(fx) = Fixture::create_or_skip().await else {
        return;
    };
    fx.sync_as("u-alice", &RunOptions::default()).await;
    fx.model.fail_rerank.store(true, Ordering::SeqCst);

    let hits = fx
        .retriever()
        .search(
            &signed_in("u-alice"),
            "rounding mode",
            &filter(&["confluence"]),
        )
        .await
        .expect("search still succeeds");
    assert!(!hits.is_empty());
    assert!(fx.model.rerank_calls.load(Ordering::SeqCst) >= 1);

    fx.db.cleanup().await;
}

#[tokio::test]
async fn a_users_second_run_cannot_start_while_their_first_is_in_flight() {
    let Some(fx) = Fixture::create_or_skip().await else {
        return;
    };
    let ingestor = Arc::new(fx.ingestor());
    let alice = UserId::new("u-alice");
    assert!(!ingestor.is_running_for(&alice));
    let report = ingestor
        .run_with(&signed_in("u-alice"), &fx.connector, &RunOptions::default())
        .await
        .expect("run");
    assert!(!report.has_errors());
    assert!(!ingestor.is_running_for(&alice), "the guard released");
    fx.db.cleanup().await;
}
