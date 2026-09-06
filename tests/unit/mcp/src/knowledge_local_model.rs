//! The local model provider: deterministic, credential-free embeddings and
//! reranking that let the knowledge bank run with real tools everywhere.
//! Embeddings are the configured width, unit-normalised, identical for
//! identical text, finite for empty text, and lexically meaningful — a query
//! sits closer to the paragraph that shares its words. Reranking orders by
//! BM25, keeps input order on ties, and truncates to `top_n`. There is no chat
//! model: `supports_chat` is false and `converse` is an error, which is what
//! lets the pipeline take its deterministic fallbacks silently.

use systemprompt_mcp_knowledge_bank::error::KnowledgeBankError;
use systemprompt_mcp_knowledge_bank::{LocalConfig, LocalProvider, ModelProvider};

const CHECKOUT_FSD: &str = "The feature is gated by the site preference expressCheckoutEnabled. The express checkout feature flag is read once per request on the server and injected into the page data, so a flag change takes effect on the next page load without a deploy.";
const CACHE_RUNBOOK: &str = "Invalidate a single page or a category subtree when the content of that page is wrong and its natural time-to-live is longer than the business can wait, for example a price correction on a product that is being promoted.";

fn provider() -> LocalProvider {
    LocalProvider::new(LocalConfig::default())
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn norm(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

#[tokio::test]
async fn an_embedding_is_deterministic_unit_length_and_the_configured_width() {
    let model = provider();
    let first = model.embed(CHECKOUT_FSD).await.expect("embed");
    let second = model.embed(CHECKOUT_FSD).await.expect("embed");
    assert_eq!(first, second, "same text, same vector");
    assert_eq!(first.len(), 1024);
    assert_eq!(model.embed_dim(), 1024);
    assert_eq!(model.embed_model(), "local.hashed-ngram-v1");
    assert!((norm(&first) - 1.0).abs() < 1e-4, "norm {}", norm(&first));
    assert!(first.iter().all(|v| v.is_finite()));
}

#[tokio::test]
async fn a_query_sits_closer_to_the_paragraph_that_shares_its_words() {
    let model = provider();
    let query = model
        .embed("express checkout feature flag")
        .await
        .expect("embed");
    let fsd = model.embed(CHECKOUT_FSD).await.expect("embed");
    let runbook = model.embed(CACHE_RUNBOOK).await.expect("embed");
    let (to_fsd, to_runbook) = (cosine(&query, &fsd), cosine(&query, &runbook));
    assert!(
        to_fsd > to_runbook,
        "checkout FSD {to_fsd} must beat the cache runbook {to_runbook}"
    );
    assert!(to_fsd > 0.0);
}

#[tokio::test]
async fn empty_and_punctuation_only_text_embed_to_a_finite_unit_vector() {
    let model = provider();
    for text in ["", "   ", "--- !!! ..."] {
        let vector = model.embed(text).await.expect("embed");
        assert_eq!(vector.len(), 1024, "{text:?}");
        assert!(vector.iter().all(|v| v.is_finite()), "{text:?}");
        assert!((norm(&vector) - 1.0).abs() < 1e-4, "{text:?}");
    }
}

#[tokio::test]
async fn the_width_and_ngram_are_configuration() {
    let model = LocalProvider::new(LocalConfig {
        embed_model: "local.test".to_owned(),
        embed_dim: 64,
        ngram: 1,
    });
    let vector = model.embed(CHECKOUT_FSD).await.expect("embed");
    assert_eq!(vector.len(), 64);
    assert_eq!(model.embed_dim(), 64);
    assert_eq!(model.embed_model(), "local.test");
    assert_eq!(model.config().ngram, 1);
}

#[tokio::test]
async fn rerank_orders_by_lexical_relevance_and_truncates_to_top_n() {
    let model = provider();
    let documents = vec![
        CACHE_RUNBOOK.to_owned(),
        "Northwind Trading is a Hamburg-based retailer of professional tools.".to_owned(),
        CHECKOUT_FSD.to_owned(),
    ];
    let ranked = model
        .rerank("express checkout feature flag", &documents, 2)
        .await
        .expect("rerank");
    assert_eq!(ranked.len(), 2, "truncated to top_n");
    assert_eq!(ranked[0].index, 2, "the checkout paragraph ranks first");
    assert!((ranked[0].relevance_score - 1.0).abs() < f64::EPSILON);
    assert!(ranked[1].relevance_score <= ranked[0].relevance_score);
    assert!(
        ranked
            .iter()
            .all(|r| (0.0..=1.0).contains(&r.relevance_score))
    );
}

#[tokio::test]
async fn rerank_keeps_input_order_on_ties_and_handles_the_empty_cases() {
    let model = provider();
    let documents = vec![
        "alpha beta".to_owned(),
        "gamma delta".to_owned(),
        "epsilon zeta".to_owned(),
    ];
    let ranked = model
        .rerank("nothing here matches", &documents, 5)
        .await
        .expect("rerank");
    let order: Vec<usize> = ranked.iter().map(|r| r.index).collect();
    assert_eq!(
        order,
        vec![0, 1, 2],
        "all scores zero: fusion order survives"
    );
    assert!(ranked.iter().all(|r| r.relevance_score == 0.0));

    let none = model.rerank("q", &[], 5).await.expect("rerank");
    assert!(none.is_empty());
    let zero = model.rerank("q", &documents, 0).await.expect("rerank");
    assert!(zero.is_empty());
}

#[tokio::test]
async fn there_is_no_chat_model() {
    let model = provider();
    assert!(!model.supports_chat());
    let error = model
        .converse("system", "user")
        .await
        .expect_err("converse is not available locally");
    assert!(
        matches!(error, KnowledgeBankError::NotConfigured(ref m) if m.contains("bedrock")),
        "{error}"
    );
}
