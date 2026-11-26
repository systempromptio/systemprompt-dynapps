//! A proxy instance: the tree and the document read through the Brain's
//! operations, and the technical pages query nothing and say where the index
//! lives. The index tables are dropped, so a query would show as a fault.

use axum::http::StatusCode;

use crate::env::Env;

#[tokio::test]
async fn in_proxy_chunks_and_ingest_show_the_notice_and_read_no_table() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::proxy(db).await;
    let id = env.invoicing.id.0;
    for uri in [
        format!("/brain/articles/{id}?lang=es&tab=chunks"),
        format!("/brain/articles/{id}?lang=es&tab=ingest"),
        format!("/brain/articles/{id}?lang=es&tab=usage"),
        "/brain/ingest".to_owned(),
        "/brain/search?q=apple".to_owned(),
        "/brain/usage".to_owned(),
    ] {
        let page = env.get(&uri, true).await;
        assert_eq!(page.status, StatusCode::OK, "{uri}");
        assert!(page.has("live at the source"), "{uri}");
        assert!(!page.has_fault_notice(), "{uri} read a table");
    }
    env.finish().await;
}

#[tokio::test]
async fn in_proxy_the_tree_and_the_document_read_through_the_operations() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::proxy(db).await;
    let tree = env.get("/brain/spaces", true).await;
    assert!(tree.has("Facturas") && tree.has("Weekend SLA"));
    let id = env.invoicing.id.0;
    let doc = env.get(&format!("/brain/articles/{id}?lang=es"), true).await;
    assert!(doc.has("Cómo emitir facturas."));
    assert!(!doc.has_fault_notice());
    env.finish().await;
}

#[tokio::test]
async fn in_proxy_the_overview_says_so_and_counts_no_chunks() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::proxy(db).await;
    let page = env.get("/brain", true).await;
    assert!(page.has("Proxy"));
    assert!(page.has("live at the source"));
    assert!(!page.has("Embeddings pending"));
    assert!(!page.has_fault_notice());
    env.finish().await;
}

#[tokio::test]
async fn in_proxy_reindex_is_refused() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::proxy(db).await;
    let id = env.invoicing.id.0;
    let page = env
        .post(&format!("/brain/articles/{id}/reindex"), "lang=es", true)
        .await;
    assert_eq!(page.status, StatusCode::CONFLICT);
    env.finish().await;
}
