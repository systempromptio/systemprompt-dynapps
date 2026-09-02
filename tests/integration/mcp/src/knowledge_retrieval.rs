//! The serving path against a live pgvector, with a fake model whose
//! embeddings and reranking a test can steer: both retrieval arms contribute
//! to one fused list (FR-RET-1); the reranker's order is served when it is
//! on (FR-RET-2) and the fusion order with zero rerank calls when it is off
//! (FR-MODEL-3); `top_k` is capped (FR-RET-6); a rebuilt index is served by
//! the same retriever with no restart and a moved version stamp (FR-RET-7);
//! a search answers while another user's sync is in flight (NFR-SELF-1); and
//! one document's embedding failure is isolated to that document
//! (FR-MODEL-5, NFR-DEGRADE-1).

use crate::knowledge_files::{Fixture, filter, signed_in, write};
use async_trait::async_trait;
use std::sync::atomic::Ordering;
use systemprompt_mcp_knowledge_bank::config::KnowledgeBankConfig;
use systemprompt_mcp_knowledge_bank::error::KnowledgeBankError;
use systemprompt_mcp_knowledge_bank::ingest::document::FetchOutcome;
use systemprompt_mcp_knowledge_bank::ingest::sources::FetchRequest;
use systemprompt_mcp_knowledge_bank::{
    FilesConnector, KnowledgeRetriever, PgRetriever, RunOptions, SearchFilter, SourceConnector,
};

const SEMANTIC: &str = "# Rounding\n\nTotals are rounded half to even so multi-currency baskets do not drift by a cent.\n";
const LEXICAL: &str = "# Glossary\n\nzanzibarium is the internal code name for the payment vault migration project.\n";
const OTHER: &str =
    "# Unrelated\n\nThe office plants are watered on Fridays by whoever is last to leave.\n";

fn adr_config(fx: &Fixture, rerank: bool, top_k: u8, multiplier: u8) -> KnowledgeBankConfig {
    let mut config = fx.config.clone();
    config.retrieval.rerank_enabled = rerank;
    config.retrieval.top_k = top_k;
    config.retrieval.candidate_multiplier = multiplier;
    config
}

fn retriever_with(fx: &Fixture, config: KnowledgeBankConfig) -> PgRetriever {
    PgRetriever::new(fx.db.raw_pool_arc(), fx.model.clone(), config)
}

async fn seed(fx: &Fixture) {
    write(fx.root.path(), "adrs/semantic.md", SEMANTIC.as_bytes());
    write(fx.root.path(), "adrs/lexical.md", LEXICAL.as_bytes());
    write(fx.root.path(), "adrs/other.md", OTHER.as_bytes());
    fx.sync_files("u-alice", &RunOptions::default()).await;
}

#[tokio::test]
async fn a_lexical_only_and_a_semantic_only_match_both_surface() {
    let Some(fx) = Fixture::create().await else {
        return;
    };
    seed(&fx).await;
    let query = "zanzibarium";
    fx.model.embed_query_as(query, SEMANTIC);
    let retriever = retriever_with(&fx, adr_config(&fx, false, 2, 1));

    let hits = retriever
        .search(
            &signed_in("u-bob"),
            query,
            &SearchFilter {
                source_types: vec!["adrs".to_owned()],
                top_k: Some(2),
            },
        )
        .await
        .expect("search");
    let ids: Vec<&str> = hits.iter().map(|h| h.doc_id.as_str()).collect();
    assert_eq!(ids.len(), 2, "{ids:?}");
    assert!(
        ids.contains(&"adrs:semantic.md"),
        "the vector arm surfaces a document that shares no word with the query: {ids:?}"
    );
    assert!(
        ids.contains(&"adrs:lexical.md"),
        "the full-text arm surfaces the only document holding the word: {ids:?}"
    );
    fx.db.cleanup().await;
}

#[tokio::test]
async fn the_reranker_order_is_served_when_enabled_and_the_fusion_order_when_disabled() {
    let Some(fx) = Fixture::create().await else {
        return;
    };
    seed(&fx).await;
    let query = "rounded half to even multi-currency";
    let fusion = retriever_with(&fx, adr_config(&fx, false, 3, 4))
        .search(&signed_in("u-bob"), query, &filter(&["adrs"]))
        .await
        .expect("search");
    assert_eq!(
        fx.model.reranks(),
        0,
        "rerank off means no rerank call (FR-MODEL-3)"
    );
    assert!(fusion.len() >= 2);
    assert_eq!(fusion[0].doc_id, "adrs:semantic.md");

    fx.model.reverse_rerank.store(true, Ordering::SeqCst);
    let reranked = retriever_with(&fx, adr_config(&fx, true, 3, 4))
        .search(&signed_in("u-bob"), query, &filter(&["adrs"]))
        .await
        .expect("search");
    assert_eq!(fx.model.reranks(), 1);
    let key = |h: &systemprompt_mcp_knowledge_bank::KnowledgeHit| (h.doc_id.clone(), h.chunk_index);
    let fused: Vec<_> = fusion.iter().map(key).collect();
    let served: Vec<_> = reranked.iter().map(key).collect();
    let mut expected = fused.clone();
    expected.reverse();
    assert_eq!(
        served, expected,
        "with rerank on, the order is the reranker's"
    );
    assert!(
        reranked[0].score > reranked[1].score,
        "scores follow the reranker too"
    );

    fx.db.cleanup().await;
}

#[tokio::test]
async fn the_result_count_never_exceeds_the_cap() {
    let Some(fx) = Fixture::create().await else {
        return;
    };
    for i in 0..25 {
        write(
            fx.root.path(),
            &format!("adrs/{i:02}.md"),
            format!(
                "# ADR-{i:02}\n\nRounding rule number {i} applies to baskets in currency {i}.\n"
            )
            .as_bytes(),
        );
    }
    fx.sync_files("u-alice", &RunOptions::default()).await;
    let retriever = retriever_with(&fx, adr_config(&fx, false, 5, 4));
    let hits = retriever
        .search(
            &signed_in("u-bob"),
            "rounding rule baskets currency",
            &SearchFilter {
                source_types: vec!["adrs".to_owned()],
                top_k: Some(200),
            },
        )
        .await
        .expect("search");
    assert_eq!(hits.len(), 20, "top_k above the cap is clamped to it");
    let defaulted = retriever
        .search(
            &signed_in("u-bob"),
            "rounding rule baskets currency",
            &SearchFilter {
                source_types: vec!["adrs".to_owned()],
                top_k: None,
            },
        )
        .await
        .expect("search");
    assert_eq!(defaulted.len(), 5, "no top_k means the configured default");

    fx.db.cleanup().await;
}

#[tokio::test]
async fn a_rebuilt_index_is_served_by_the_same_retriever_without_a_restart() {
    let Some(fx) = Fixture::create().await else {
        return;
    };
    fx.sync_files("u-alice", &RunOptions::default()).await;
    let retriever = fx.retriever();
    let version_before = retriever.index_stats().await.expect("stats").version;
    let before = retriever
        .search(
            &signed_in("u-bob"),
            "half to even drift by a cent",
            &filter(&["adrs"]),
        )
        .await
        .expect("search");
    assert!(before.iter().all(|h| h.doc_id != "adrs:semantic.md"));

    write(fx.root.path(), "adrs/semantic.md", SEMANTIC.as_bytes());
    fx.sync_files("u-alice", &RunOptions::default()).await;

    let after = retriever
        .search(
            &signed_in("u-bob"),
            "half to even drift by a cent",
            &filter(&["adrs"]),
        )
        .await
        .expect("search");
    assert_eq!(
        after.first().map(|h| h.doc_id.as_str()),
        Some("adrs:semantic.md")
    );
    let version_after = retriever.index_stats().await.expect("stats").version;
    assert_ne!(version_before, version_after, "the version stamp moved");

    fx.db.cleanup().await;
}

// A connector that blocks every source until released, to hold a sync
// mid-flight.
struct Gate {
    released: std::sync::atomic::AtomicBool,
    open: tokio::sync::Notify,
}

#[async_trait]
impl SourceConnector for Gate {
    async fn fetch(&self, _request: &FetchRequest<'_>) -> Result<FetchOutcome, KnowledgeBankError> {
        while !self.released.load(Ordering::SeqCst) {
            self.open.notified().await;
        }
        Ok(FetchOutcome::default())
    }
}

#[tokio::test]
async fn a_search_answers_while_another_users_sync_is_in_flight() {
    let Some(fx) = Fixture::create().await else {
        return;
    };
    fx.sync_files("u-alice", &RunOptions::default()).await;
    let gate = std::sync::Arc::new(Gate {
        released: std::sync::atomic::AtomicBool::new(false),
        open: tokio::sync::Notify::new(),
    });
    let ingestor = std::sync::Arc::new(fx.ingestor());
    let running = {
        let ingestor = ingestor.clone();
        let gate = gate.clone();
        tokio::spawn(async move {
            ingestor
                .run_with(&signed_in("u-bob"), gate.as_ref(), &RunOptions::default())
                .await
        })
    };
    for _ in 0..200 {
        if ingestor.is_running_for(&signed_in("u-bob").user_id) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
    assert!(ingestor.is_running_for(&signed_in("u-bob").user_id));

    let hits = fx
        .retriever()
        .search(
            &signed_in("u-alice"),
            "banker's rounding CHF",
            &filter(&["adrs"]),
        )
        .await
        .expect("serving does not wait for ingestion");
    assert!(!hits.is_empty());

    gate.released.store(true, Ordering::SeqCst);
    gate.open.notify_waiters();
    running
        .await
        .expect("join")
        .expect("the held sync completes");

    fx.db.cleanup().await;
}

#[tokio::test]
async fn a_document_whose_embedding_fails_is_counted_failed_while_its_siblings_are_written() {
    let Some(fx) = Fixture::create().await else {
        return;
    };
    write(
        fx.root.path(),
        "adrs/poison.md",
        b"# Poison\n\nThis body carries the FAIL-EMBED marker.\n",
    );
    fx.model.fail_embeds_containing("FAIL-EMBED");
    let report = fx
        .ingestor()
        .run_with(
            &signed_in("u-alice"),
            &FilesConnector,
            &RunOptions {
                sources: vec!["adrs".to_owned()],
                ..RunOptions::default()
            },
        )
        .await
        .expect("the run completes");
    let adrs = &report.sources[0];
    assert_eq!(adrs.failed, 1, "{adrs:?}");
    assert_eq!(
        adrs.written, 1,
        "the sibling ADR is still written: {adrs:?}"
    );
    assert!(
        adrs.error.is_none(),
        "a document failure is not a source failure"
    );
    assert_eq!(fx.db.count("knowledge_documents").await, 1);

    fx.db.cleanup().await;
}
