//! REQ-052 (P0) enforcement, proven against a live Postgres.
//!
//! The claim under test is the acceptance criterion itself: unauthorized
//! documents are never returned, permission changes propagate, and a caller
//! with no grants sees nothing. `list_sources` is the probe because it runs
//! the same `knowledge_document_grants` join as the search arms while needing
//! no Bedrock call, so the access-control behaviour is tested without a
//! network dependency or an API key.

use crate::common::TempDb;
use std::sync::Arc;
use systemprompt::identifiers::UserId;
use systemprompt_mcp_knowledge_bank::config::{BedrockConfig, KnowledgeBankConfig};
use systemprompt_mcp_knowledge_bank::{BedrockClient, KnowledgeRetriever, PgRetriever, Principal};

fn retriever(db: &TempDb) -> PgRetriever {
    let config = KnowledgeBankConfig::default();
    // Why: the client is constructed but never called — list_sources is pure
    // SQL. A real key would make this test depend on the network and on a
    // credential no CI run should need.
    let bedrock = BedrockClient::new(BedrockConfig::default(), "unused".to_owned())
        .expect("the default region is inside the EU");
    PgRetriever::new(db.raw_pool_arc(), Arc::new(bedrock), config)
}

fn principal(user_id: &str, roles: &[&str]) -> Principal {
    Principal {
        user_id: UserId::new(user_id),
        roles: roles.iter().map(|r| (*r).to_owned()).collect(),
        attributes: std::collections::BTreeMap::new(),
    }
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

    let ids: Vec<&str> = sources.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(
        ids,
        vec!["confluence"],
        "a document granted only to role hr must not surface for an engineer"
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
        sources.is_empty(),
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
        sources.is_empty(),
        "ingestion that could not determine a document's permissions must fail closed"
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
    assert_eq!(before.len(), 1);

    db.revoke("doc-eng", "user", "u-eng").await;

    let after = retriever(&db)
        .list_sources(&caller)
        .await
        .expect("listing succeeds");
    assert!(
        after.is_empty(),
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

    assert_eq!(sources.len(), 1);
    assert_eq!(
        sources[0].doc_count,
        Some(1),
        "reporting the index total would disclose how much exists in a category \
         the caller cannot read"
    );

    db.cleanup().await;
}
