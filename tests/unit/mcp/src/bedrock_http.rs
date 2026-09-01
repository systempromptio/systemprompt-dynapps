//! The Bedrock client, against a mocked `bedrock-runtime`.
//!
//! Every embedding, chunk decision and rerank in the knowledge bank goes
//! through this one client, and none of it had ever run in a test: the
//! endpoint is derived from the configured region, so exercising it meant
//! calling AWS. `BedrockProvider::with_endpoint` is the seam core gives its
//! own provider drivers for exactly this, and the fixtures below follow the
//! same shape — a fresh `MockServer` per case, pointed at by the provider.
//!
//! What is asserted is the contract the rest of the crate depends on: an
//! embedding of the configured width or an error (never a short vector, which
//! pgvector would reject at insert time), the retry classification that
//! decides whether a failure is worth trying again, and the rerank ordering
//! that the retrieval degradation contract keys on.

use serde_json::{Value, json};
use systemprompt_mcp_knowledge_bank::config::BedrockConfig;
use systemprompt_mcp_knowledge_bank::model::{BedrockProvider, ModelProvider, assert_eu_region};
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

const EMBED_DIM: u16 = 8;

fn config() -> BedrockConfig {
    BedrockConfig {
        embed_dim: EMBED_DIM,
        ..BedrockConfig::default()
    }
}

fn provider_for(server: &MockServer) -> BedrockProvider {
    BedrockProvider::new(config(), "test-key".to_owned())
        .expect("the default config is an EU region")
        .with_endpoint(server.uri())
}

// `InvokeModel` for any model id, answering `body` every time.
async fn invoke_returns(status: u16, body: Value) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(r"/model/.+/invoke"))
        .respond_with(ResponseTemplate::new(status).set_body_json(body))
        .mount(&server)
        .await;
    server
}

async fn invoke_fails(status: u16, detail: &str) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(r"/model/.+/invoke"))
        .respond_with(ResponseTemplate::new(status).set_body_string(detail.to_owned()))
        .mount(&server)
        .await;
    server
}

fn embedding_of(width: usize) -> Value {
    json!({ "embedding": (0..width).map(|i| f64::from(u32::try_from(i).unwrap_or(0)) / 10.0).collect::<Vec<_>>() })
}

#[tokio::test]
async fn an_embedding_comes_back_at_the_configured_width() {
    let server = invoke_returns(200, embedding_of(EMBED_DIM as usize)).await;

    let vector = provider_for(&server)
        .embed("the deployment runbook")
        .await
        .expect("the embedding call succeeds");

    assert_eq!(vector.len(), usize::from(EMBED_DIM));
}

#[tokio::test]
async fn a_short_vector_is_refused_rather_than_stored() {
    let server = invoke_returns(200, embedding_of(3)).await;

    let error = provider_for(&server)
        .embed("text")
        .await
        .expect_err("a vector of the wrong width is refused");

    // Why: pgvector's column is fixed-width, so a short vector fails at insert
    // time with a message about the column rather than about the model. Being
    // refused here names the actual cause.
    let message = error.to_string();
    assert!(message.contains('3'), "the width it got: {message}");
    assert!(
        message.contains(&EMBED_DIM.to_string()),
        "and the width it wanted: {message}"
    );
}

#[tokio::test]
async fn a_response_with_no_embedding_field_is_an_error_not_an_empty_vector() {
    let server = invoke_returns(200, json!({ "message": "quota exceeded" })).await;

    let error = provider_for(&server)
        .embed("text")
        .await
        .expect_err("a response without the array is refused");

    assert!(
        error.to_string().contains("embedding"),
        "the message names the field that was missing: {error}"
    );
}

#[tokio::test]
async fn a_client_error_is_fatal_and_reported_once() {
    // 403 will not become 200 on a retry, so the client must not spend the
    // attempt budget on it.
    let server = invoke_fails(403, "not authorized to invoke this model").await;

    let error = provider_for(&server)
        .embed("text")
        .await
        .expect_err("a 403 surfaces");

    let message = error.to_string();
    assert!(message.contains("403"), "{message}");
    assert!(
        !message.contains("attempts"),
        "a fatal failure is not retried: {message}"
    );
    assert_eq!(
        server.received_requests().await.map(|r| r.len()),
        Some(1),
        "and the call was made exactly once"
    );
}

// Why `start_paused`: the client backs off up to 20s between ten attempts, so
// these two cases would otherwise cost over a minute of wall clock each. With
// a paused clock the sleeps resolve instantly and the retry behaviour is
// unchanged — the assertion is on the number of calls, not on their timing.
#[tokio::test(start_paused = true)]
async fn a_throttle_is_retried_before_it_is_given_up_on() {
    let server = invoke_fails(429, "too many requests").await;

    let error = provider_for(&server)
        .embed("text")
        .await
        .expect_err("the retries are eventually exhausted");

    let message = error.to_string();
    assert!(
        message.contains("attempts"),
        "the exhaustion says how many were made: {message}"
    );
    let attempts = server
        .received_requests()
        .await
        .map(|r| r.len())
        .unwrap_or_default();
    assert!(
        attempts > 1,
        "a 429 is transient and must be retried, got {attempts} call(s)"
    );
}

#[tokio::test(start_paused = true)]
async fn a_server_error_is_retried_too() {
    let server = invoke_fails(503, "service unavailable").await;

    let _ = provider_for(&server).embed("text").await.expect_err("gives up");

    let attempts = server
        .received_requests()
        .await
        .map(|r| r.len())
        .unwrap_or_default();
    assert!(attempts > 1, "5xx is transient, got {attempts} call(s)");
}

#[tokio::test]
async fn a_malformed_body_is_fatal_because_a_retry_cannot_fix_it() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(r"/model/.+/invoke"))
        .respond_with(ResponseTemplate::new(200).set_body_string("this is not json"))
        .mount(&server)
        .await;

    let _ = provider_for(&server).embed("text").await.expect_err("refused");

    assert_eq!(
        server.received_requests().await.map(|r| r.len()),
        Some(1),
        "a body that will not parse is not worth asking for again"
    );
}

#[tokio::test]
async fn rerank_returns_the_documents_in_the_order_the_model_ranked_them() {
    let server = invoke_returns(
        200,
        json!({
            "results": [
                { "index": 2, "relevance_score": 0.91 },
                { "index": 0, "relevance_score": 0.44 }
            ]
        }),
    )
    .await;
    let documents = vec![
        "a note about billing".to_owned(),
        "an unrelated page".to_owned(),
        "the deployment runbook".to_owned(),
    ];

    let ranked = provider_for(&server)
        .rerank("how do I deploy", &documents, 2)
        .await
        .expect("the rerank call succeeds");

    assert_eq!(ranked.len(), 2);
    assert_eq!(ranked[0].index, 2, "the model's own order is preserved");
    assert!((ranked[0].relevance_score - 0.91).abs() < f64::EPSILON);
    assert_eq!(ranked[1].index, 0);
}

#[tokio::test]
async fn reranking_nothing_asks_bedrock_nothing() {
    let server = invoke_returns(200, json!({ "results": [] })).await;

    let ranked = provider_for(&server)
        .rerank("query", &[], 5)
        .await
        .expect("an empty rerank is a success");

    assert!(ranked.is_empty());
    assert_eq!(
        server.received_requests().await.map(|r| r.len()),
        Some(0),
        "an empty candidate set is answered locally, not billed"
    );
}

#[tokio::test]
async fn a_rerank_result_missing_its_score_is_refused() {
    let server = invoke_returns(200, json!({ "results": [{ "index": 0 }] })).await;

    let error = provider_for(&server)
        .rerank("query", &["one".to_owned()], 1)
        .await
        .expect_err("an incomplete result is refused");

    assert!(
        error.to_string().contains("relevance_score"),
        "the message names what was missing: {error}"
    );
}

#[tokio::test]
async fn the_residency_guard_still_refuses_a_non_eu_region() {
    // The endpoint override must not become a way around REQ-037: the guard
    // runs in `new`, before any endpoint is chosen.
    assert!(assert_eu_region("eu-central-1").is_ok());
    let error = BedrockProvider::new(
        BedrockConfig {
            region: "us-east-1".to_owned(),
            ..config()
        },
        "test-key".to_owned(),
    )
    .expect_err("a US region is refused");
    assert!(error.to_string().contains("us-east-1"), "{error}");
}
