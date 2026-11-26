//! How the search ranks: the fusion of the two arms, the reranker and its
//! breaker, the fingerprint check, one passage per document in the language
//! asked for, the confidence threshold, the query cache and the `where`
//! filter.

use std::sync::Arc;

use dynbrain::brain::Brain;
use dynbrain::brain::args::SearchArgs;
use dynbrain::brain::fault::BrainFault;
use dynbrain::search::confidence::MatchConfidence;
use dynbrain::search::filter::PropertyFilter;
use dynbrain::search::rerank::{FusionReason, ScoreSource};
use dynbrain::search::{RetrievalMode, RetrievalRequest, RetrievalSettings, SearchScope};
use serde_json::json;

use crate::fixture::{Fixture, user};
use crate::search_support::{
    QueryEmbedder, ScriptedReranker, ask, axis, brain, chunk, chunk_by, doc, hybrid, retriever,
    with_properties,
};

fn titles(found: &dynbrain::brain::found::SearchResults) -> Vec<&str> {
    found.results.iter().map(|hit| hit.title.as_str()).collect()
}

#[tokio::test]
async fn a_chunk_in_both_arms_outranks_one_in_the_text_arm_only() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let both = doc(&f, space.id, "both", &[("es", "Both", "x")]).await;
    let lexical = doc(&f, space.id, "lexical", &[("es", "Lexical", "x")]).await;
    chunk(
        &f,
        &both,
        ("es", 0),
        "apple among unrelated filler words here",
        Some(&axis(&[0])),
    )
    .await;
    chunk(
        &f,
        &lexical,
        ("es", 0),
        "apple apple apple apple apple",
        None,
    )
    .await;
    let found = hybrid(&f)
        .search(&user("alice"), &ask("apple"))
        .await
        .expect("search");
    assert_eq!(
        titles(&found),
        ["Both", "Lexical"],
        "the fused score, not the text rank alone"
    );
    let first = found.results[0].score.unwrap_or_default();
    assert!((first - (1.0 / 60.0 + 1.0 / 61.0)).abs() < 1e-5, "{first}");
    assert_eq!(found.verdict.confidence, MatchConfidence::Unrated);
    f.finish().await;
}

#[tokio::test]
async fn a_foreign_fingerprint_switches_the_vector_arm_off() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let lexical = doc(&f, space.id, "lexical", &[("es", "Lexical", "x")]).await;
    let stale = doc(&f, space.id, "stale", &[("es", "Stale", "x")]).await;
    chunk(&f, &lexical, ("es", 0), "banana bread recipe", None).await;
    let vector = axis(&[0]);
    chunk_by(
        &f,
        &stale,
        ("es", 0),
        "completely unrelated text about tectonic plates",
        Some((vector.as_slice(), "other:old-model:1536")),
    )
    .await;
    let found = hybrid(&f)
        .search(&user("alice"), &ask("banana"))
        .await
        .expect("search");
    assert_eq!(found.mode, RetrievalMode::Hybrid);
    assert_eq!(
        titles(&found),
        ["Lexical"],
        "the stale vector never entered the pool"
    );
    f.finish().await;
}

async fn two_documents(f: &Fixture) {
    let space = f.space("handbook").await;
    let first = doc(f, space.id, "first", &[("es", "First", "x")]).await;
    let second = doc(f, space.id, "second", &[("es", "Second", "x")]).await;
    chunk(
        f,
        &first,
        ("es", 0),
        "alpha first passage",
        Some(&axis(&[0])),
    )
    .await;
    chunk(
        f,
        &second,
        ("es", 0),
        "beta second passage",
        Some(&axis(&[0, 1])),
    )
    .await;
}

fn everyone(who: &str) -> SearchScope {
    SearchScope {
        reader: user(who),
        space: None,
        lang: None,
        filter: PropertyFilter::default(),
    }
}

#[tokio::test]
async fn a_failing_reranker_leaves_unrated_and_the_fusion_order() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    two_documents(&f).await;
    let plain = hybrid(&f)
        .search(&user("alice"), &ask("passage"))
        .await
        .expect("search");
    let failing = ScriptedReranker::failing(503);
    let native = brain(
        &f,
        retriever(
            &f,
            Some(QueryEmbedder::on(&[0])),
            Some(Arc::clone(&failing)),
            RetrievalSettings::default(),
        ),
    );
    let found = native
        .search(&user("alice"), &ask("passage"))
        .await
        .expect("search");
    assert_eq!(titles(&found), titles(&plain), "the fused order is kept");
    assert_eq!(titles(&found), ["First", "Second"]);
    assert_eq!(found.verdict.confidence, MatchConfidence::Unrated);
    assert!(found.verdict.confidence_note.is_some());
    assert_eq!(failing.calls(), 1);
    f.finish().await;
}

#[tokio::test]
async fn the_breaker_opens_on_a_failure_on_the_wire_and_not_on_a_refusal() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    two_documents(&f).await;
    let scope = everyone("alice");
    let request = RetrievalRequest {
        query: "passage",
        scope: &scope,
        limit: 10,
        prefer_lang: None,
    };
    let down = ScriptedReranker::failing(503);
    let search = retriever(
        &f,
        Some(QueryEmbedder::on(&[0])),
        Some(Arc::clone(&down)),
        RetrievalSettings::default(),
    );
    let first = search.retrieve(&request).await.expect("retrieve");
    assert_eq!(first.trace.reason, Some(FusionReason::Error));
    let second = search.retrieve(&request).await.expect("retrieve");
    assert_eq!(second.trace.reason, Some(FusionReason::BreakerOpen));
    assert_eq!(second.trace.scoring, ScoreSource::Fusion);
    assert_eq!(down.calls(), 1, "a written-off provider is not called");
    let refusing = ScriptedReranker::failing(401);
    let search = retriever(
        &f,
        Some(QueryEmbedder::on(&[0])),
        Some(Arc::clone(&refusing)),
        RetrievalSettings::default(),
    );
    search.retrieve(&request).await.expect("retrieve");
    let again = search.retrieve(&request).await.expect("retrieve");
    assert_eq!(again.trace.reason, Some(FusionReason::Error));
    assert_eq!(
        refusing.calls(),
        2,
        "a refused key keeps being tried and logged"
    );
    f.finish().await;
}

#[tokio::test]
async fn a_working_reranker_orders_by_its_scores_and_rates_the_best() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    two_documents(&f).await;
    let scored = ScriptedReranker::scoring(&[("beta", 0.93), ("alpha", 0.2)]);
    let native = brain(
        &f,
        retriever(
            &f,
            Some(QueryEmbedder::on(&[0])),
            Some(scored),
            RetrievalSettings::default(),
        ),
    );
    let found = native
        .search(&user("alice"), &ask("passage"))
        .await
        .expect("search");
    assert_eq!(titles(&found), ["Second", "First"]);
    assert_eq!(found.verdict.confidence, MatchConfidence::High);
    assert_eq!(found.verdict.top_score, Some(0.93));
    let weak = ScriptedReranker::scoring(&[("beta", 0.41)]);
    let native = brain(
        &f,
        retriever(
            &f,
            Some(QueryEmbedder::on(&[0])),
            Some(weak),
            RetrievalSettings::default(),
        ),
    );
    let found = native
        .search(&user("alice"), &ask("passage"))
        .await
        .expect("search");
    assert_eq!(found.verdict.confidence, MatchConfidence::Low);
    assert!(
        found
            .verdict
            .confidence_note
            .as_deref()
            .is_some_and(|note| note.contains("scored 0.41, below the 0.70"))
    );
    f.finish().await;
}

#[tokio::test]
async fn the_threshold_is_a_setting() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    two_documents(&f).await;
    let strict = RetrievalSettings {
        weak_match_threshold: 0.95,
        ..RetrievalSettings::default()
    };
    let native = brain(
        &f,
        retriever(
            &f,
            Some(QueryEmbedder::on(&[0])),
            Some(ScriptedReranker::scoring(&[("beta", 0.93)])),
            strict,
        ),
    );
    let found = native
        .search(&user("alice"), &ask("passage"))
        .await
        .expect("search");
    assert_eq!(found.verdict.confidence, MatchConfidence::Low);
    f.finish().await;
}

#[tokio::test]
async fn one_passage_per_document_in_the_language_asked_for() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let twin = doc(
        &f,
        space.id,
        "twin",
        &[("es", "Copias", "x"), ("en", "Backups", "x")],
    )
    .await;
    chunk(
        &f,
        &twin,
        ("es", 0),
        "copias de seguridad diarias",
        Some(&axis(&[0])),
    )
    .await;
    chunk(
        &f,
        &twin,
        ("en", 0),
        "daily backups of everything",
        Some(&axis(&[0, 1])),
    )
    .await;
    let native = hybrid(&f);
    let english = SearchArgs {
        lang: Some("en".to_owned()),
        ..ask("backups")
    };
    let found = native
        .search(&user("alice"), &english)
        .await
        .expect("search");
    assert_eq!(
        titles(&found),
        ["Backups"],
        "one result, the English variant"
    );
    let spanish = SearchArgs {
        lang: Some("es".to_owned()),
        ..ask("backups")
    };
    let found = native
        .search(&user("alice"), &spanish)
        .await
        .expect("search");
    assert_eq!(titles(&found), ["Copias"]);
    let detected = native
        .search(
            &user("alice"),
            &ask("the backups of the team and the copies"),
        )
        .await
        .expect("search");
    assert_eq!(titles(&detected), ["Backups"], "the query reads as English");
    f.finish().await;
}

#[tokio::test]
async fn a_repeated_query_is_embedded_once_and_unused_ones_age_out() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    two_documents(&f).await;
    let embedder = QueryEmbedder::on(&[0]);
    let native = brain(
        &f,
        retriever(
            &f,
            Some(Arc::clone(&embedder)),
            None,
            RetrievalSettings::default(),
        ),
    );
    native
        .search(&user("alice"), &ask("Alpha  Passage"))
        .await
        .expect("search");
    native
        .search(&user("alice"), &ask("alpha passage"))
        .await
        .expect("search");
    assert_eq!(embedder.calls(), 1, "the normalised query hits the cache");
    let hits: i32 = sqlx::query_scalar("SELECT hits FROM dynbrain_query_cache")
        .fetch_one(f.pool())
        .await
        .expect("the cache row");
    assert_eq!(hits, 2);
    sqlx::query(
        "UPDATE dynbrain_query_cache SET last_used = CURRENT_TIMESTAMP - INTERVAL '31 days'",
    )
    .execute(f.pool())
    .await
    .expect("age the entry");
    native
        .search(&user("alice"), &ask("beta"))
        .await
        .expect("search");
    let left: Vec<String> = sqlx::query_scalar("SELECT query_text FROM dynbrain_query_cache")
        .fetch_all(f.pool())
        .await
        .expect("the cache rows");
    assert_eq!(left, ["beta"]);
    f.finish().await;
}

#[tokio::test]
async fn without_an_embedder_the_search_matches_keywords_unrated() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    doc(
        &f,
        space.id,
        "a",
        &[("es", "Retention", "keep 100% of the logs_daily")],
    )
    .await;
    doc(
        &f,
        space.id,
        "b",
        &[("es", "Other", "keep some of the logs")],
    )
    .await;
    let native = brain(&f, retriever(&f, None, None, RetrievalSettings::default()));
    let found = native
        .search(&user("alice"), &ask("100%"))
        .await
        .expect("search");
    assert_eq!(found.mode, RetrievalMode::Keyword);
    assert_eq!(titles(&found), ["Retention"], "% is matched literally");
    assert_eq!(found.verdict.confidence, MatchConfidence::Unrated);
    assert_eq!(found.results[0].score, None);
    f.finish().await;
}

#[tokio::test]
async fn the_where_filter_keeps_documents_with_the_value_or_without_the_property() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let decided = doc(&f, space.id, "decided", &[]).await;
    let draft = doc(&f, space.id, "draft", &[]).await;
    let bare = doc(&f, space.id, "bare", &[]).await;
    with_properties(&f, &decided, &json!({"status": "decided"})).await;
    with_properties(&f, &draft, &json!({"status": "draft"})).await;
    with_properties(&f, &bare, &json!({})).await;
    for article in [&decided, &draft, &bare] {
        chunk(&f, article, ("es", 0), "common words", Some(&axis(&[0]))).await;
    }
    let native = hybrid(&f);
    let filtered = |filter: serde_json::Value| SearchArgs {
        filter: filter.as_object().cloned(),
        ..ask("common")
    };
    let found = native
        .search(&user("alice"), &filtered(json!({"status": "decided"})))
        .await
        .expect("search");
    assert_eq!(titles(&found), ["Doc decided"]);
    let unset = native
        .search(&user("alice"), &filtered(json!({"status": false})))
        .await
        .expect("search");
    assert_eq!(titles(&unset), ["Doc bare"]);
    let keywords = brain(&f, retriever(&f, None, None, RetrievalSettings::default()));
    let by_words = SearchArgs {
        filter: json!({"status": "draft"}).as_object().cloned(),
        ..ask("body")
    };
    let by_words = keywords
        .search(&user("alice"), &by_words)
        .await
        .expect("search");
    assert_eq!(titles(&by_words), ["Doc draft"]);
    let unknown = native
        .search(&user("alice"), &filtered(json!({"owner": "ana"})))
        .await;
    assert!(matches!(unknown, Err(BrainFault::Property(_))));
    f.finish().await;
}
