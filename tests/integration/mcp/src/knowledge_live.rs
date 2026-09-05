//! The whole pipeline against real Bedrock and, as the caller, real
//! Confluence: sync the configured space with header-style credentials, then
//! search it as that user and check the citations point at the site. Costs
//! money and needs credentials, so it self-skips unless
//! `KNOWLEDGE_BANK_BEDROCK_KEY`, `KNOWLEDGE_BANK_ATLASSIAN_EMAIL` and
//! `KNOWLEDGE_BANK_ATLASSIAN_API_TOKEN` are all set (optionally
//! `KNOWLEDGE_BANK_ATLASSIAN_BASE_URL` and `KNOWLEDGE_BANK_CONFLUENCE_SPACE`,
//! default EAICOMM). Run it when adopting a new key, region, model or site,
//! not on every `just test`.
//!
//! A second, cheaper check needs only `KNOWLEDGE_BANK_PLAIN_DATABASE_URL` — a
//! Postgres *without* pgvector — and proves the boot refuses it with the
//! message the operator will see.

use crate::common::TempDb;
use axum::http::{HeaderMap, HeaderValue};
use std::sync::Arc;
use systemprompt::identifiers::UserId;
use systemprompt_mcp_knowledge_bank::config::KnowledgeBankConfig;
use systemprompt_mcp_knowledge_bank::downstream::{
    AtlassianCredentials, HEADER_API_TOKEN, HEADER_EMAIL,
};
use systemprompt_mcp_knowledge_bank::error::KnowledgeBankError;
use systemprompt_mcp_knowledge_bank::model::build_provider;
use systemprompt_mcp_knowledge_bank::{
    Ingestor, KnowledgeRetriever, PgRetriever, Principal, RunOptions, SearchFilter, schema,
};

const KEY_ENV: &str = "KNOWLEDGE_BANK_BEDROCK_KEY";
const EMAIL_ENV: &str = "KNOWLEDGE_BANK_ATLASSIAN_EMAIL";
const TOKEN_ENV: &str = "KNOWLEDGE_BANK_ATLASSIAN_API_TOKEN";
const BASE_URL_ENV: &str = "KNOWLEDGE_BANK_ATLASSIAN_BASE_URL";
const SPACE_ENV: &str = "KNOWLEDGE_BANK_CONFLUENCE_SPACE";
const PLAIN_DB_ENV: &str = "KNOWLEDGE_BANK_PLAIN_DATABASE_URL";

fn user() -> Principal {
    Principal {
        user_id: UserId::new("u-live"),
        roles: vec!["user".to_owned()],
        attributes: std::collections::BTreeMap::new(),
    }
}

#[tokio::test]
async fn the_real_pipeline_syncs_confluence_as_the_caller_and_answers_with_citations() {
    let (Ok(key), Ok(email), Ok(token)) = (
        std::env::var(KEY_ENV),
        std::env::var(EMAIL_ENV),
        std::env::var(TOKEN_ENV),
        // skip-ok: no live Confluence credentials on this machine
    ) else {
        return;
    };
    let base_url = std::env::var(BASE_URL_ENV)
        .unwrap_or_else(|_| "https://astounddigital.atlassian.net".to_owned());
    let space = std::env::var(SPACE_ENV).unwrap_or_else(|_| "EAICOMM".to_owned());
    let Some(db) = TempDb::create().await else {
        return;
    };
    db.create_knowledge_schema().await;
    let config = KnowledgeBankConfig::parse(&format!(
        "atlassian: {{base_url: {base_url}}}\nsources:\n  confluence: {{kind: confluence, strategy: script, spaces: [{space}]}}\n"
    ))
    .expect("config parses");
    let model = build_provider(&config.model, Some(key)).expect("bedrock provider builds");

    // The credentials take the exact path the server uses: request headers.
    let mut headers = HeaderMap::new();
    headers.insert(HEADER_EMAIL, HeaderValue::from_str(&email).expect("email"));
    headers.insert(
        HEADER_API_TOKEN,
        HeaderValue::from_str(&token).expect("token"),
    );
    let credentials =
        AtlassianCredentials::from_headers(&headers, &config.atlassian).expect("credentials");

    let ingestor = Ingestor::new(
        db.raw_pool_arc(),
        Arc::clone(&model),
        Arc::new(config.clone()),
    );
    let report = ingestor
        .run(&user(), Some(credentials), &RunOptions::default())
        .await
        .expect("live sync succeeds");
    assert!(!report.has_errors(), "{report:?}");
    let written: usize = report.sources.iter().map(|s| s.written).sum();
    assert!(written > 0, "the space has pages the caller can read");

    let retriever = PgRetriever::new(db.raw_pool_arc(), model, config);
    let sources = retriever.list_sources(&user()).await.expect("listing");
    let confluence = sources
        .iter()
        .find(|s| s.id == "confluence")
        .expect("listed");
    assert_eq!(confluence.doc_count, Some(written as u64));
    assert!(confluence.last_synced.is_some());

    let hits = retriever
        .search(
            &user(),
            "what is this space about",
            &SearchFilter {
                source_types: Vec::new(),
                top_k: Some(3),
            },
        )
        .await
        .expect("live search succeeds");
    assert!(!hits.is_empty());
    assert!(
        hits.iter().all(|h| h.uri.starts_with(&base_url)),
        "citations point at the site: {:?}",
        hits.iter().map(|h| &h.uri).collect::<Vec<_>>()
    );

    let stranger = retriever
        .search(
            &Principal {
                user_id: UserId::new("u-other"),
                roles: vec!["admin".to_owned()],
                attributes: std::collections::BTreeMap::new(),
            },
            "what is this space about",
            &SearchFilter {
                source_types: Vec::new(),
                top_k: Some(3),
            },
        )
        .await
        .expect("search");
    assert!(
        stranger.is_empty(),
        "even an admin who has not synced holds no grant"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn a_database_without_pgvector_is_refused_at_boot_with_the_operator_message() {
    // skip-ok: no pgvector-free Postgres configured on this machine
    let Ok(url) = std::env::var(PLAIN_DB_ENV) else {
        return;
    };
    let pool = sqlx::PgPool::connect(&url)
        .await
        .expect("plain database reachable");
    let described = schema::describe_database_url(&url);
    let err = schema::require(&pool, &described)
        .await
        .expect_err("a plain Postgres must be refused");
    assert!(matches!(err, KnowledgeBankError::NotConfigured(_)));
    let text = err.to_string();
    assert!(text.contains("has no pgvector; it is required"), "{text}");
    assert!(text.contains(&described), "{text}");
    assert!(text.contains("docs/install/knowledge-db.md"), "{text}");
}
