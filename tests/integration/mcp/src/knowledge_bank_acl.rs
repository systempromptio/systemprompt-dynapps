//! REQ-052 (P0) enforcement, proven against a live Postgres.
//!
//! The claim under test is the acceptance criterion itself: unauthorized
//! documents are never returned, permission changes propagate, and a caller
//! with no grants sees nothing. `list_sources` is the probe because it runs
//! the same `knowledge_document_grants` join as the search arms while needing
//! no model call. Every configured category is listed — the catalogue is
//! configuration — but the count is the caller's own.

use crate::common::TempDb;
use crate::fake_model::FakeModel;
use std::sync::Arc;
use systemprompt::identifiers::UserId;
use systemprompt_mcp_knowledge_bank::config::KnowledgeBankConfig;
use systemprompt_mcp_knowledge_bank::{
    KnowledgeRetriever, KnowledgeSource, PgRetriever, Principal,
};

const CONFIG: &str = "
atlassian: {base_url: https://example.atlassian.net}
sources:
  hr_policies: {kind: confluence, strategy: script, spaces: [HR]}
  confluence:  {kind: confluence, strategy: script, spaces: [ENG]}
";

fn retriever(db: &TempDb) -> PgRetriever {
    let config = KnowledgeBankConfig::parse(CONFIG).expect("test config parses");
    PgRetriever::new(db.raw_pool_arc(), Arc::new(FakeModel::default()), config)
}

fn principal(user_id: &str, roles: &[&str]) -> Principal {
    Principal {
        user_id: UserId::new(user_id),
        roles: roles.iter().map(|r| (*r).to_owned()).collect(),
        attributes: std::collections::BTreeMap::new(),
    }
}

fn readable(sources: &[KnowledgeSource]) -> Vec<(&str, u64)> {
    sources
        .iter()
        .filter(|s| s.available)
        .map(|s| (s.id.as_str(), s.doc_count.unwrap_or(0)))
        .collect()
}

#[tokio::test]
async fn a_caller_sees_only_sources_it_has_a_grant_for() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    db.create_knowledge_schema().await;
    db.insert_document("doc-hr", "hr_policies").await;
    db.insert_document("doc-eng", "confluence").await;
    db.grant("doc-hr", "role", "hr").await;
    db.grant("doc-eng", "role", "engineer").await;

    let sources = retriever(&db)
        .list_sources(&principal("u-eng", &["engineer"]))
        .await
        .expect("listing succeeds");

    assert_eq!(sources.len(), 2, "every configured category is listed");
    assert_eq!(
        readable(&sources),
        vec![("confluence", 1)],
        "a document granted only to role hr must not count for an engineer"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn a_caller_with_no_grants_sees_nothing() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    db.create_knowledge_schema().await;
    db.insert_document("doc-hr", "hr_policies").await;
    db.grant("doc-hr", "role", "hr").await;

    let sources = retriever(&db)
        .list_sources(&principal("u-stranger", &[]))
        .await
        .expect("listing succeeds");

    assert!(
        readable(&sources).is_empty(),
        "an unrecognised caller must fail closed, not see the whole index"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn a_document_with_no_grants_is_reachable_by_nobody() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    db.create_knowledge_schema().await;
    db.insert_document("doc-orphan", "confluence").await;

    let sources = retriever(&db)
        .list_sources(&principal("u-admin", &["admin"]))
        .await
        .expect("listing succeeds");

    assert!(
        readable(&sources).is_empty(),
        "ingestion that could not translate a document's permissions must fail closed"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn revoking_a_grant_propagates_on_the_next_call() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    db.create_knowledge_schema().await;
    db.insert_document("doc-eng", "confluence").await;
    db.grant("doc-eng", "user", "u-eng").await;

    let caller = principal("u-eng", &[]);
    let before = retriever(&db)
        .list_sources(&caller)
        .await
        .expect("listing succeeds");
    assert_eq!(readable(&before), vec![("confluence", 1)]);

    db.revoke("doc-eng", "user", "u-eng").await;

    let after = retriever(&db)
        .list_sources(&caller)
        .await
        .expect("listing succeeds");
    assert!(
        readable(&after).is_empty(),
        "a revoked grant must take effect on the next call, with no cache to flush"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn document_counts_are_the_callers_own_not_the_indexs() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    db.create_knowledge_schema().await;
    for id in ["doc-a", "doc-b", "doc-c"] {
        db.insert_document(id, "confluence").await;
    }
    db.grant("doc-a", "user", "u-eng").await;

    let sources = retriever(&db)
        .list_sources(&principal("u-eng", &[]))
        .await
        .expect("listing succeeds");

    assert_eq!(
        readable(&sources),
        vec![("confluence", 1)],
        "reporting the index total would disclose how much exists in a category \
         the caller cannot read"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn the_schema_applies_idempotently() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    db.create_knowledge_schema().await;
    db.create_knowledge_schema().await;
    let stats = retriever(&db)
        .index_stats()
        .await
        .expect("stats on an empty index");
    assert_eq!((stats.documents, stats.chunks), (0, 0));
    assert!(
        stats.version.is_none(),
        "nothing has been indexed, so no version is stamped"
    );
    db.cleanup().await;
}
