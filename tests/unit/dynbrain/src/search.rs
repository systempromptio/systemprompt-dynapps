//! The pure parts of the search: fusion, deduplication, confidence, the
//! reranking stage and its breaker, the query-cache key, the language of a
//! query and the `where` filter.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use dynbrain::ids::{ArticleId, ChunkId, SpaceId};
use dynbrain::properties::PropertyError;
use dynbrain::search::Passage;
use dynbrain::search::confidence::{MatchConfidence, NOTE_UNRATED, low_note, verdict};
use dynbrain::search::dedup::dedup;
use dynbrain::search::filter::PropertyFilter;
use dynbrain::search::fusion::fuse;
use dynbrain::search::lang::detect;
use dynbrain::search::query::cache_key;
use dynbrain::search::rerank::{
    FusionReason, RerankBreaker, RerankFailure, Reranker, ScoreSource, cooldown, rerank,
};
use serde_json::{Map, Value, json};

fn passage(article: i64, lang: &str, content: &str, score: f64) -> Passage {
    Passage {
        chunk: Some(ChunkId(article * 10 + i64::from(lang == "en"))),
        article: ArticleId(article),
        space: SpaceId(1),
        lang: lang.to_owned(),
        title: format!("Doc {article}"),
        description: String::new(),
        content: content.to_owned(),
        score: Some(score),
    }
}

fn close(left: f64, right: f64) -> bool {
    (left - right).abs() < 1e-12
}

#[test]
fn fusion_adds_both_arms_and_ranks_by_the_sum() {
    let fused = fuse(
        &[ChunkId(1), ChunkId(2), ChunkId(3)],
        &[ChunkId(3), ChunkId(1), ChunkId(4)],
    );
    let order: Vec<i64> = fused.iter().map(|(chunk, _)| chunk.0).collect();
    assert_eq!(order, [1, 3, 2, 4]);
    assert!(close(fused[0].1, 1.0 / 60.0 + 1.0 / 61.0));
    assert!(close(fused[1].1, 1.0 / 62.0 + 1.0 / 60.0));
    assert!(close(fused[2].1, 1.0 / 61.0));
    assert!(close(fused[3].1, 1.0 / 62.0));
}

#[test]
fn fusion_keeps_ties_in_the_order_first_seen() {
    let fused = fuse(&[ChunkId(7)], &[ChunkId(5)]);
    let order: Vec<i64> = fused.iter().map(|(chunk, _)| chunk.0).collect();
    assert_eq!(order, [7, 5], "the vector arm's chunk was seen first");
    assert!(fuse(&[], &[]).is_empty());
}

#[test]
fn dedup_keeps_one_passage_per_document_in_rank_order() {
    let kept = dedup(
        vec![
            passage(1, "es", "uno dos tres", 0.9),
            passage(2, "es", "cuatro cinco", 0.8),
            passage(1, "es", "seis siete", 0.7),
        ],
        None,
    );
    let articles: Vec<i64> = kept.iter().map(|kept| kept.article.0).collect();
    assert_eq!(articles, [1, 2]);
    assert_eq!(kept[0].content, "uno dos tres");
}

#[test]
fn dedup_drops_a_near_duplicate_of_another_document() {
    let original = "the quick brown fox jumps over the lazy dog near the river bank today";
    let kept = dedup(
        vec![
            passage(1, "es", original, 0.9),
            passage(2, "en", &format!("{original} indeed"), 0.8),
            passage(3, "es", "something else entirely", 0.7),
        ],
        None,
    );
    let articles: Vec<i64> = kept.iter().map(|kept| kept.article.0).collect();
    assert_eq!(articles, [1, 3], "a Jaccard of 0.85 or more is a duplicate");
}

#[test]
fn dedup_prefers_the_asked_language_without_reordering_documents() {
    let hits = vec![
        passage(1, "es", "hola mundo", 0.9),
        passage(1, "en", "hello world", 0.8),
        passage(2, "es", "otra cosa", 0.5),
    ];
    let kept = dedup(hits.clone(), Some("en"));
    assert_eq!(
        kept[0].lang, "en",
        "the English variant speaks for document 1"
    );
    assert_eq!(kept[0].article, ArticleId(1));
    assert_eq!(
        kept[1].article,
        ArticleId(2),
        "a Spanish-only document still comes back"
    );
    assert_eq!(dedup(hits, Some("es"))[0].lang, "es");
}

#[test]
fn confidence_is_unrated_unless_the_reranker_scored() {
    let fused = verdict(Some(0.016), ScoreSource::Fusion, 0.70);
    assert_eq!(fused.confidence, MatchConfidence::Unrated);
    assert_eq!(fused.confidence_note.as_deref(), Some(NOTE_UNRATED));
    let empty = verdict(None, ScoreSource::CrossEncoder, 0.70);
    assert_eq!(empty.confidence, MatchConfidence::Unrated);
    assert_eq!(empty.top_score, None);
}

#[test]
fn confidence_splits_the_best_score_at_the_threshold() {
    let high = verdict(Some(0.912_34), ScoreSource::CrossEncoder, 0.70);
    assert_eq!(high.confidence, MatchConfidence::High);
    assert_eq!(high.confidence_note, None);
    assert!(close(high.top_score.unwrap_or_default(), 0.9123));
    let low = verdict(Some(0.41), ScoreSource::CrossEncoder, 0.70);
    assert_eq!(low.confidence, MatchConfidence::Low);
    assert_eq!(low.confidence_note, Some(low_note(0.41, 0.70)));
    assert!(low_note(0.41, 0.70).contains("scored 0.41, below the 0.70"));
    let json = serde_json::to_value(&high).expect("serialize");
    assert_eq!(json["confidence"], "high");
    assert!(json.get("confidence_note").is_none());
}

#[test]
fn a_zero_threshold_switches_the_signal_off() {
    let off = verdict(Some(0.1), ScoreSource::CrossEncoder, 0.0);
    assert_eq!(off.confidence, MatchConfidence::High);
}

#[test]
fn the_breaker_opens_for_a_minute_and_doubles_up_to_fifteen() {
    let minutes: Vec<u64> = (1..=6)
        .map(|failures| cooldown(failures).as_secs() / 60)
        .collect();
    assert_eq!(minutes, [1, 2, 4, 8, 15, 15]);
    let breaker = RerankBreaker::default();
    let now = Instant::now();
    assert_eq!(
        breaker.remaining("cohere", "https://a", now),
        Duration::ZERO
    );
    assert_eq!(
        breaker.fail("cohere", "https://a", now),
        Duration::from_mins(1)
    );
    assert_eq!(
        breaker.remaining("cohere", "https://a", now),
        Duration::from_mins(1)
    );
    assert_eq!(
        breaker.remaining("cohere", "https://b", now),
        Duration::ZERO,
        "another endpoint of the same provider is its own provider"
    );
    assert_eq!(
        breaker.fail("cohere", "https://a", now),
        Duration::from_mins(2)
    );
}

#[test]
fn the_breaker_lets_a_trial_through_after_its_window_and_closes_on_success() {
    let breaker = RerankBreaker::default();
    let now = Instant::now();
    breaker.fail("cohere", "https://a", now);
    let later = now + Duration::from_secs(61);
    assert_eq!(
        breaker.remaining("cohere", "https://a", later),
        Duration::ZERO
    );
    breaker.succeed("cohere", "https://a");
    assert_eq!(
        breaker.fail("cohere", "https://a", later),
        Duration::from_mins(1),
        "a success forgets the earlier failures"
    );
}

#[test]
fn refusals_do_not_open_the_breaker_but_failures_on_the_wire_do() {
    for code in [400, 401, 403, 422] {
        assert!(!RerankFailure::Status(code).opens_breaker(), "{code}");
    }
    for code in [404, 410, 429, 500, 502, 503] {
        assert!(RerankFailure::Status(code).opens_breaker(), "{code}");
    }
    assert!(RerankFailure::TimedOut(Duration::from_secs(5)).opens_breaker());
}

#[derive(Debug, Default)]
struct Failing {
    calls: AtomicUsize,
}

#[async_trait]
impl Reranker for Failing {
    fn provider(&self) -> &'static str {
        "failing"
    }

    fn endpoint(&self) -> &'static str {
        "test://failing"
    }

    async fn rerank(
        &self,
        _query: &str,
        _documents: &[String],
    ) -> Result<Vec<(usize, f64)>, RerankFailure> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Err(RerankFailure::Status(503))
    }
}

#[tokio::test]
async fn reranking_keeps_the_fused_order_and_says_why() {
    let fused = vec![passage(1, "es", "a", 0.03), passage(2, "es", "b", 0.02)];
    let breaker = RerankBreaker::default();
    let (kept, trace) = rerank(None, &breaker, "q", fused.clone()).await;
    assert_eq!(kept, fused);
    assert_eq!(trace.scoring, ScoreSource::Fusion);
    assert_eq!(trace.reason, Some(FusionReason::NoProvider));
    let failing = Failing::default();
    let (kept, trace) = rerank(Some(&failing), &breaker, "q", fused.clone()).await;
    assert_eq!(kept, fused);
    assert_eq!(trace.reason, Some(FusionReason::Error));
    assert_eq!(trace.provider.as_deref(), Some("failing"));
    let (_, trace) = rerank(Some(&failing), &breaker, "q", fused.clone()).await;
    assert_eq!(trace.reason, Some(FusionReason::BreakerOpen));
    assert_eq!(failing.calls.load(Ordering::SeqCst), 1);
    let (kept, trace) = rerank(Some(&failing), &breaker, "q", Vec::new()).await;
    assert!(kept.is_empty());
    assert_eq!(trace.reason, Some(FusionReason::NoHits));
}

#[test]
fn the_query_cache_key_is_sha1_of_provider_model_and_the_normalised_query() {
    let key = cache_key("openai:text-embedding-3-large:1536", "  Backup   POLICY ");
    assert_eq!(key.provider, "openai");
    assert_eq!(key.model, "text-embedding-3-large");
    assert_eq!(
        key.hash,
        cache_key("openai:text-embedding-3-large:1536", "backup policy").hash
    );
    assert_eq!(
        key.hash, "0df8bb66500e83849dcb8677106850c6bfddd5c3",
        "sha1(\"openai:text-embedding-3-large:backup policy\")"
    );
    assert_ne!(
        key.hash,
        cache_key("openai:other:1536", "backup policy").hash
    );
}

#[test]
fn the_language_of_a_query_follows_dynbrain() {
    let all: Vec<String> = ["es", "en", "fr", "nl", "de", "pt", "it"]
        .iter()
        .map(|code| (*code).to_owned())
        .collect();
    let cases = [
        (
            "This is the documentation of the product and it can be used for that.",
            Some("en"),
        ),
        (
            "Esta es la documentacion del producto y se puede usar para eso con el equipo.",
            Some("es"),
        ),
        (
            "Ceci est la documentation du produit et il peut etre utilise pour cela avec vous.",
            Some("fr"),
        ),
        (
            "Dit is de documentatie van het product en het kan worden gebruikt voor dat met je.",
            Some("nl"),
        ),
        (
            "Das ist die Dokumentation von dem Produkt und es kann fur das mit der werden.",
            Some("de"),
        ),
        ("", None),
        ("12345 6789", None),
        ("the zqx wvu", None),
        ("the of der und", None),
    ];
    for (text, expected) in cases {
        assert_eq!(detect(text, &all).as_deref(), expected, "{text}");
    }
    let spanish_only = vec!["es".to_owned()];
    assert_eq!(
        detect("the of and to for with this that which", &spanish_only),
        None
    );
    assert_eq!(detect("the of and to", &[]), None);
}

#[test]
fn the_where_filter_takes_declared_properties_only_and_false_means_not_set() {
    let declared: BTreeSet<String> = ["owner".to_owned(), "status".to_owned()].into();
    let asked: Map<String, Value> = json!({"status": "decided", "owner": false})
        .as_object()
        .cloned()
        .expect("an object");
    let filter = PropertyFilter::parse(Some(&asked), &declared).expect("a filter");
    assert_eq!(filter.unset, ["owner"]);
    let props = |value: Value| value.as_object().cloned().expect("an object");
    assert!(filter.admits(&props(json!({"status": "decided"}))));
    assert!(filter.admits(&props(json!({"status": "decided", "owner": ""}))));
    assert!(!filter.admits(&props(json!({"status": "decided", "owner": "ana"}))));
    assert!(!filter.admits(&props(json!({"status": "draft"}))));
    let unknown: Map<String, Value> = json!({"color": "red"})
        .as_object()
        .cloned()
        .expect("an object");
    let error = PropertyFilter::parse(Some(&unknown), &declared).expect_err("undeclared");
    assert!(matches!(error, PropertyError::NotDeclared { .. }));
    assert_eq!(
        error.to_string(),
        "'color' is not a metadata field of this documentation. Declared: owner, status."
    );
}
