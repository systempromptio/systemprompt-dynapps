//! The Chunks and Ingest tabs, the global queue, Reindex and Retry.

use axum::http::StatusCode;

use crate::env::{Env, FINGERPRINT};

#[tokio::test]
async fn the_chunks_tab_shows_each_chunk_as_the_index_holds_it() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let id = env.invoicing.id.0;
    let page = env
        .get(&format!("/brain/articles/{id}?lang=es&tab=chunks"), true)
        .await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has("Facturas › Rectificativas"), "heading_path");
    assert!(page.has("simple"), "fts_config");
    assert!(page.has(FINGERPRINT), "the fingerprint");
    assert!(page.has("heading_path") && page.has("Embedding"));
    env.finish().await;
}

#[tokio::test]
async fn opening_a_chunk_shows_its_prefix_and_its_whole_text() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let id = env.invoicing.id.0;
    let page = env
        .get(&format!("/brain/articles/{id}?lang=es&tab=chunks"), true)
        .await;
    assert!(page.has("Open chunk 0"));
    assert!(page.has("Contextual prefix"));
    assert!(page.has("Facturas"), "the prefix carries the document title");
    assert!(page.has("apple credit notes to correct an invoice"));
    env.finish().await;
}

#[tokio::test]
async fn a_chunk_whose_embedding_failed_shows_error() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let id = env.credit.id.0;
    let page = env
        .get(&format!("/brain/articles/{id}?lang=es&tab=chunks"), true)
        .await;
    assert!(page.has("sp-badge--err"), "the embedding badge is an error");
    assert!(page.has("pending credit note text"));
    env.finish().await;
}

#[tokio::test]
async fn the_ingest_tab_shows_the_steps_and_the_last_error() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    sqlx::query(
        "UPDATE dynbrain_index_job SET state = 'failed', attempts = 3, last_error = 'OpenAI 429 rate limit'
          WHERE article_id = $1 AND lang = 'es'",
    )
    .bind(env.credit.id.0)
    .execute(&*env.db.pool)
    .await
    .expect("fail a job");
    let id = env.credit.id.0;
    let page = env
        .get(&format!("/brain/articles/{id}?lang=es&tab=ingest"), true)
        .await;
    for step in ["Saved", "Chunked", "Synopsis", "Embeddings", "Searchable"] {
        assert!(page.has(step), "the step {step}");
    }
    assert!(page.has("OpenAI 429 rate limit"));
    assert!(page.has("Retry") && page.has("Reindex"));
    env.finish().await;
}

#[tokio::test]
async fn the_global_queue_lists_the_jobs_with_their_errors_and_filters_by_state() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    sqlx::query(
        "UPDATE dynbrain_index_job SET state = 'failed', attempts = 5, last_error = 'embedding quota'
          WHERE article_id = $1 AND lang = 'es'",
    )
    .bind(env.credit.id.0)
    .execute(&*env.db.pool)
    .await
    .expect("fail a job");
    let all = env.get("/brain/ingest", true).await;
    assert!(all.has("embedding quota") && all.has("Facturas"));
    let failed = env.get("/brain/ingest?state=failed", true).await;
    assert!(failed.has("embedding quota"));
    assert!(!failed.has("Weekend SLA"), "pending jobs are filtered out");
    env.finish().await;
}

#[tokio::test]
async fn reindex_queues_the_document_and_leaves_an_audit_row() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let id = env.invoicing.id.0;
    sqlx::query("UPDATE dynbrain_index_job SET state = 'done' WHERE article_id = $1")
        .bind(id)
        .execute(&*env.db.pool)
        .await
        .expect("settle the jobs");
    let page = env
        .post(&format!("/brain/articles/{id}/reindex"), "lang=es", true)
        .await;
    assert_eq!(page.status, StatusCode::SEE_OTHER);
    assert!(page.location.contains("tab=ingest"));
    let state = env
        .sql_for(
            "SELECT state FROM dynbrain_index_job WHERE article_id = $1 AND lang = 'es'",
            id,
        )
        .await;
    assert_eq!(state, "pending");
    let action = env
        .sql_text("SELECT action || ':' || actor_id FROM dynbrain_admin_audit")
        .await;
    assert_eq!(action, "reindex:root");
    env.finish().await;
}

#[tokio::test]
async fn reindex_of_a_language_the_article_lacks_is_404() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let id = env.invoicing.id.0;
    let page = env
        .post(&format!("/brain/articles/{id}/reindex"), "lang=fr", true)
        .await;
    assert_eq!(page.status, StatusCode::NOT_FOUND);
    env.finish().await;
}

#[tokio::test]
async fn retry_requeues_a_failed_job() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    sqlx::query(
        "UPDATE dynbrain_index_job SET state = 'failed', attempts = 5, last_error = 'boom'
          WHERE article_id = $1 AND lang = 'es'",
    )
    .bind(env.credit.id.0)
    .execute(&*env.db.pool)
    .await
    .expect("fail a job");
    let job: i64 = env
        .sql_for(
            "SELECT id::text FROM dynbrain_index_job WHERE article_id = $1 AND lang = 'es'",
            env.credit.id.0,
        )
        .await
        .parse()
        .expect("a job id");
    let page = env
        .post(&format!("/brain/ingest/{job}/retry"), "", true)
        .await;
    assert_eq!(page.status, StatusCode::SEE_OTHER);
    let row = env
        .sql_for(
            "SELECT state || ':' || attempts || ':' || COALESCE(last_error, '') FROM dynbrain_index_job WHERE id = $1",
            job,
        )
        .await;
    assert_eq!(row, "pending:0:");
    env.finish().await;
}

#[tokio::test]
async fn retry_of_a_job_that_did_not_fail_is_404() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let job = env
        .sql_text("SELECT id::text FROM dynbrain_index_job ORDER BY id LIMIT 1")
        .await;
    let page = env
        .post(&format!("/brain/ingest/{job}/retry"), "", true)
        .await;
    assert_eq!(page.status, StatusCode::NOT_FOUND);
    env.finish().await;
}

#[tokio::test]
async fn the_files_usage_and_history_tabs_answer() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let id = env.invoicing.id.0;
    for tab in ["files", "usage", "history"] {
        let page = env
            .get(&format!("/brain/articles/{id}?lang=es&tab={tab}"), true)
            .await;
        assert_eq!(page.status, StatusCode::OK, "{tab}");
        assert!(!page.has_fault_notice(), "{tab}");
    }
    let history = env
        .get(&format!("/brain/articles/{id}?lang=es&tab=history"), true)
        .await;
    assert!(history.has("Revisions") && history.has("Facturas"));
    env.finish().await;
}
