//! The whole pipeline against a running instance: read Confluence through
//! the caller's *connected* Atlassian account via the gateway, index it with
//! the shipped local model, then search it as that user and check the
//! citations point at the site. Needs a live `just start` and a bearer for a
//! user who has linked Atlassian on `/admin/profile`, so it self-skips unless
//! `KNOWLEDGE_BANK_LIVE_BEARER` is set (optionally
//! `KNOWLEDGE_BANK_GATEWAY_URL`, default `http://localhost:8080`, `KNOWLEDGE_BANK_ATLASSIAN_BASE_URL` and
//! `KNOWLEDGE_BANK_CONFLUENCE_SPACE`, default EAICOMM). Run it when adopting
//! a new site or gateway, not on every `just test`.
//!
//! A second, cheaper check needs only `KNOWLEDGE_BANK_PLAIN_DATABASE_URL` — a
//! Postgres *without* pgvector — and proves the boot refuses it with the
//! message the operator will see.

use crate::common::TempDb;
use std::sync::Arc;
use systemprompt::identifiers::{AgentName, ContextId, SessionId, TraceId, UserId};
use systemprompt::models::execution::context::RequestContext;
use systemprompt_mcp_knowledge_bank::config::KnowledgeBankConfig;
use systemprompt_mcp_knowledge_bank::error::KnowledgeBankError;
use systemprompt_mcp_knowledge_bank::model::build_provider;
use systemprompt_mcp_knowledge_bank::{
    AtlassianMcp, Ingestor, KnowledgeRetriever, PgRetriever, Principal, RunOptions, SearchFilter,
    schema,
};

const BEARER_ENV: &str = "KNOWLEDGE_BANK_LIVE_BEARER";
const GATEWAY_ENV: &str = "KNOWLEDGE_BANK_GATEWAY_URL";
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

fn context() -> RequestContext {
    RequestContext::new(
        SessionId::new("kb-live-session"),
        TraceId::new("kb-live-trace"),
        ContextId::new_unchecked("00000000-0000-4000-8000-00000000a11e"),
        AgentName::new("kb-live"),
    )
}

#[tokio::test]
async fn the_real_pipeline_syncs_confluence_through_the_connected_account_and_cites_the_site() {
    // skip-ok: no live gateway bearer on this machine
    let Ok(bearer) = std::env::var(BEARER_ENV) else {
        return;
    };
    let gateway = std::env::var(GATEWAY_ENV).unwrap_or_else(|_| "http://localhost:8080".to_owned());
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
    let model = build_provider(&config.model, None).expect("the local provider builds");

    // The connection takes the exact path the server uses: the gateway's
    // Atlassian route with the caller's bearer.
    let atlassian = AtlassianMcp::connect_gateway(
        &format!("{}/api/v1/mcp/atlassian/mcp", gateway.trim_end_matches('/')),
        &bearer,
        &context(),
        &config.atlassian,
    )
    .await
    .expect("the connected account is reachable through the gateway");

    let ingestor = Ingestor::new(
        db.raw_pool_arc(),
        Arc::clone(&model),
        Arc::new(config.clone()),
    );
    let report = ingestor
        .run(&user(), Some(atlassian), &RunOptions::default())
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
                project_ids: Vec::new(),
                exclude_shared: false,
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
                project_ids: Vec::new(),
                exclude_shared: false,
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
