//! The search lab: the scores of each arm, deduplication, the answer
//! context, "Run as" and its audit, and the `explain` order.

use axum::http::StatusCode;
use dynbrain::search::filter::PropertyFilter;
use dynbrain::search::{Outcome, RetrievalRequest, SearchScope};

use crate::env::{Env, user};

#[tokio::test]
async fn the_lab_shows_the_rank_and_score_of_each_arm_and_the_answer_context() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let page = env.get("/brain/search?q=apple&rerank=1", true).await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(
        page.has("· 1.00"),
        "the cosine of a chunk on the query's axis"
    );
    assert!(page.has("Keyword · rank · ts_rank"));
    assert!(page.has("RRF k=60"));
    assert!(page.has("[1]"), "a kept chunk is numbered");
    assert!(page.has("characters per document"));
    assert!(page.has("Answer context"));
    assert!(page.has("apple"), "the context carries the text");
    env.finish().await;
}

#[tokio::test]
async fn the_lab_strikes_what_deduplication_drops_and_says_why() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let page = env.get("/brain/search?q=apple&rerank=1", true).await;
    assert!(page.has("Same document as a better chunk"));
    assert!(page.has("sp-brain-drop"));
    env.finish().await;
}

#[tokio::test]
async fn run_as_filters_by_the_permissions_of_that_person() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let admin = env.get("/brain/search?q=apple&rerank=1", true).await;
    assert!(admin.has("[2]"), "the admin finds both spaces");
    let bob = env
        .get("/brain/search?q=apple&rerank=1&run_as=bob", true)
        .await;
    assert_eq!(bob.status, StatusCode::OK);
    assert!(!bob.has("[2]"), "bob reads the handbook only");
    assert!(bob.has("No access for bob (filtered in SQL)"));
    assert!(
        bob.has("Weekend SLA"),
        "the struck row names what he cannot read"
    );
    env.finish().await;
}

#[tokio::test]
async fn run_as_leaves_an_audit_row_naming_both_people() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    env.get("/brain/search?q=apple&run_as=bob", true).await;
    let audit = env
        .sql_text("SELECT action || ':' || actor_id || ':' || subject_id || ':' || detail FROM dynbrain_admin_audit")
        .await;
    assert_eq!(audit, "run_as:root:bob:apple");
    env.finish().await;
}

#[tokio::test]
async fn a_plain_lab_run_is_audited_too() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    env.get("/brain/search?q=apple", true).await;
    let audit = env
        .sql_text("SELECT action || ':' || actor_id FROM dynbrain_admin_audit")
        .await;
    assert_eq!(audit, "search_lab:root");
    env.finish().await;
}

#[tokio::test]
async fn the_lab_run_is_not_counted_as_usage() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    env.get("/brain/search?q=apple", true).await;
    let logged = env
        .sql_text("SELECT count(*)::text FROM dynbrain_search_log")
        .await;
    assert_eq!(logged, "0");
    env.finish().await;
}

#[tokio::test]
async fn the_explain_order_is_the_order_of_search() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let scope = SearchScope {
        reader: user("root"),
        space: None,
        lang: None,
        filter: PropertyFilter::default(),
    };
    let request = RetrievalRequest {
        query: "apple",
        scope: &scope,
        limit: 10,
        prefer_lang: None,
    };
    let found = env.retriever.retrieve(&request).await.expect("search");
    let explained = env
        .retriever
        .explain(&request, true)
        .await
        .expect("explain");
    let searched: Vec<_> = found.passages.iter().map(|passage| passage.chunk).collect();
    let kept: Vec<_> = explained.kept.iter().map(|passage| passage.chunk).collect();
    assert_eq!(kept, searched);
    let shown: Vec<_> = explained
        .rows
        .iter()
        .filter(|row| row.outcome == Outcome::Kept)
        .map(|row| row.passage.chunk)
        .collect();
    assert_eq!(shown.len(), searched.len());
    assert!(shown.iter().all(|chunk| searched.contains(chunk)));
    assert!(
        explained
            .rows
            .iter()
            .any(|row| row.outcome == Outcome::SameDocument),
        "two chunks of one document: one is dropped"
    );
    let top = explained.rows.first().expect("a candidate");
    assert!(
        top.vector.is_some() && top.text.is_some(),
        "both arms scored it"
    );
    env.finish().await;
}
