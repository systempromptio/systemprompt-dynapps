//! Live Bedrock reachability, and the residency guard that fences it.
//!
//! The reachability tests cost money and need a credential, so they self-skip
//! unless `KNOWLEDGE_BANK_BEDROCK_KEY` is set — run them when adopting a new
//! key or region, not on every `just test`. The region guard needs neither and
//! always runs: it is the only thing standing between an edit to
//! `services/knowledge-bank/config.yaml` and Astound's Confluence content
//! being embedded in Virginia, because these calls never touch the AI gateway
//! where REQ-037 residency routing lives.

use systemprompt_mcp_knowledge_bank::bedrock::embed_text;
use systemprompt_mcp_knowledge_bank::config::BedrockConfig;
use systemprompt_mcp_knowledge_bank::{BedrockClient, assert_eu_region};

const KEY_ENV: &str = "KNOWLEDGE_BANK_BEDROCK_KEY";

#[test]
fn the_configured_default_region_is_inside_the_eu() {
    assert!(assert_eu_region(&BedrockConfig::default().region).is_ok());
}

#[test]
fn a_us_region_is_refused_with_a_reason_that_names_the_requirement() {
    let err = assert_eu_region("us-east-1").expect_err("a US region must be refused");
    let message = err.to_string();
    assert!(message.contains("us-east-1"));
    assert!(
        message.contains("REQ-037"),
        "the refusal must say which requirement it is protecting"
    );
}

#[test]
fn a_client_cannot_be_built_for_a_non_eu_region() {
    let config = BedrockConfig {
        region: "us-east-1".to_owned(),
        ..BedrockConfig::default()
    };
    assert!(
        BedrockClient::new(config, "unused".to_owned()).is_err(),
        "the guard must run at construction, not at first call"
    );
}

#[tokio::test]
async fn titan_returns_a_vector_of_the_configured_width() {
    let Ok(key) = std::env::var(KEY_ENV) else {
        return;
    };
    let config = BedrockConfig::default();
    let expected = usize::from(config.embed_dim);
    let client = BedrockClient::new(config, key).expect("eu-central-1 client builds");

    let embedding = embed_text(&client, "deployment runbook")
        .await
        .expect("titan embeds in eu-central-1");

    assert_eq!(embedding.len(), expected);
}

#[tokio::test]
async fn cohere_reranks_the_more_relevant_document_first() {
    let Ok(key) = std::env::var(KEY_ENV) else {
        return;
    };
    let client =
        BedrockClient::new(BedrockConfig::default(), key).expect("eu-central-1 client builds");
    let documents = vec![
        "The quarterly finance report for the retail division.".to_owned(),
        "How to roll back a failed deployment and verify health.".to_owned(),
    ];

    let ranked = systemprompt_mcp_knowledge_bank::bedrock::rerank(
        &client,
        "how do I roll back a deployment",
        &documents,
        2,
    )
    .await
    .expect("cohere reranks in eu-central-1");

    assert_eq!(
        ranked.first().map(|r| r.index),
        Some(1),
        "the deployment document must outrank the finance one"
    );
}
