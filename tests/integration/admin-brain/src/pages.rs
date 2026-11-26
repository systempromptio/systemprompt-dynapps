//! Who may enter, and the tree, the document and the overview.

use axum::http::StatusCode;

use crate::env::Env;

fn pages(env: &Env) -> Vec<String> {
    let id = env.invoicing.id.0;
    vec![
        "/brain".to_owned(),
        "/brain/spaces".to_owned(),
        format!("/brain/articles/{id}"),
        "/brain/search?q=apple".to_owned(),
        "/brain/ingest".to_owned(),
        "/brain/usage".to_owned(),
    ]
}

#[tokio::test]
async fn every_page_answers_200_to_an_admin() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    for uri in pages(&env) {
        let page = env.get(&uri, true).await;
        assert_eq!(page.status, StatusCode::OK, "{uri}");
        assert!(!page.has_fault_notice(), "{uri} could not load something");
    }
    env.finish().await;
}

#[tokio::test]
async fn every_page_answers_403_to_someone_who_is_not_an_admin() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    for uri in pages(&env) {
        let page = env.get(&uri, false).await;
        assert_eq!(page.status, StatusCode::FORBIDDEN, "{uri}");
    }
    env.finish().await;
}

#[tokio::test]
async fn the_three_writes_answer_403_to_someone_who_is_not_an_admin() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let id = env.invoicing.id.0;
    let reindex = env
        .post(&format!("/brain/articles/{id}/reindex"), "lang=es", false)
        .await;
    let retry = env.post("/brain/ingest/1/retry", "", false).await;
    let grant = env
        .post(
            &format!("/brain/articles/{id}/access"),
            "kind=user&subject=carol&level=read&lang=es",
            false,
        )
        .await;
    for page in [reindex, retry, grant] {
        assert_eq!(page.status, StatusCode::FORBIDDEN);
    }
    let audited = env
        .sql_text("SELECT count(*)::text FROM dynbrain_admin_audit")
        .await;
    assert_eq!(audited, "0", "a refused write leaves no audit row");
    env.finish().await;
}

#[tokio::test]
async fn the_tree_shows_every_space_and_article() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let page = env.get("/brain/spaces", true).await;
    for name in [
        "Space handbook",
        "Space support",
        "Facturas",
        "Rectificativas",
        "Weekend SLA",
    ] {
        assert!(page.has(name), "the tree lacks {name}");
    }
    assert!(page.has("<details"), "the branches fold");
    env.finish().await;
}

#[tokio::test]
async fn an_article_opens_in_the_language_asked_for() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let id = env.invoicing.id.0;
    let spanish = env.get(&format!("/brain/articles/{id}?lang=es"), true).await;
    assert!(spanish.has("Cómo emitir facturas."));
    let english = env.get(&format!("/brain/articles/{id}?lang=en"), true).await;
    assert!(english.has("How to issue invoices."));
    assert!(english.has("tab=content"), "the languages and tabs are links");
    env.finish().await;
}

#[tokio::test]
async fn a_language_the_article_lacks_shows_the_default_with_a_notice() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let id = env.invoicing.id.0;
    let page = env.get(&format!("/brain/articles/{id}?lang=fr"), true).await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has("no variant in that language"));
    assert!(page.has("Cómo emitir facturas."));
    env.finish().await;
}

#[tokio::test]
async fn an_unknown_article_is_404() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let page = env.get("/brain/articles/987654", true).await;
    assert_eq!(page.status, StatusCode::NOT_FOUND);
    env.finish().await;
}

#[tokio::test]
async fn the_overview_counts_the_index_and_names_the_providers() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let page = env.get("/brain", true).await;
    assert!(page.has("Native"));
    assert!(page.has("Documents indexed"));
    assert!(page.has("Embeddings pending"));
    assert!(page.has("text-embedding-3-large"));
    assert!(page.has("rerank-v3.5"));
    assert!(page.has("0.70"), "the confidence threshold");
    assert!(page.has("Space handbook"));
    env.finish().await;
}
