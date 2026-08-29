//! REQ-046: >70% precision@5 on an agreed benchmark set.
//!
//! The harness is here and runnable; the corpus is not. The indexing account
//! currently sees a single Confluence page and gets 401 from Jira, so there is
//! nothing to measure against — see `rag-access-request.local.md`. Rather than
//! report a meaningless number from an empty index, the harness refuses to
//! score while the fixture is still marked `placeholder`, and says why.
//!
//! To run it for real: replace `fixtures/knowledge-benchmark.json` with agreed
//! questions and their answering `doc_id`s, drop `placeholder`, index the
//! corpus, then set `KNOWLEDGE_BANK_BEDROCK_KEY` (search embeds every question,
//! so this costs money and needs a credential).

use serde_json::Value;

const FIXTURE: &str = include_str!("../fixtures/knowledge-benchmark.json");
const KEY_ENV: &str = "KNOWLEDGE_BANK_BEDROCK_KEY";

fn fixture() -> Value {
    serde_json::from_str(FIXTURE).expect("the benchmark fixture is valid JSON")
}

pub fn precision_at_k(returned: &[String], relevant: &[String], k: usize) -> f64 {
    if k == 0 {
        return 0.0;
    }
    let hits = returned
        .iter()
        .take(k)
        .filter(|doc_id| relevant.contains(doc_id))
        .count();
    hits as f64 / k as f64
}

#[test]
fn the_benchmark_fixture_declares_a_threshold_matching_the_requirement() {
    let fixture = fixture();
    let threshold = fixture["threshold"]
        .as_f64()
        .expect("threshold is a number");
    assert!(
        (threshold - 0.7).abs() < f64::EPSILON,
        "REQ-046 states >70% precision@5; the fixture must not quietly relax it"
    );
}

#[test]
fn precision_at_five_counts_only_the_first_five_results() {
    let returned: Vec<String> = (0..10).map(|i| format!("doc-{i}")).collect();
    let relevant = vec!["doc-0".to_owned(), "doc-9".to_owned()];

    assert!((precision_at_k(&returned, &relevant, 5) - 0.2).abs() < f64::EPSILON);
}

#[test]
fn precision_is_zero_when_nothing_relevant_is_returned() {
    let returned = vec!["doc-a".to_owned(), "doc-b".to_owned()];
    let relevant = vec!["doc-z".to_owned()];

    assert!(precision_at_k(&returned, &relevant, 5).abs() < f64::EPSILON);
}

#[tokio::test]
async fn the_corpus_benchmark_refuses_to_score_placeholder_cases() {
    let fixture = fixture();
    if fixture["placeholder"].as_bool() == Some(true) {
        eprintln!(
            "REQ-046 benchmark not run: fixtures/knowledge-benchmark.json is still a \
             placeholder. Blocked on Confluence space access and a Jira application \
             licence — see rag-access-request.local.md."
        );
        return;
    }
    assert!(
        std::env::var(KEY_ENV).is_ok(),
        "a real benchmark run needs {KEY_ENV} to embed each question"
    );
    let cases = fixture["cases"].as_array().expect("cases is an array");
    assert!(
        cases.iter().all(|case| {
            case["relevant_doc_ids"]
                .as_array()
                .is_some_and(|ids| !ids.is_empty())
        }),
        "every non-placeholder case must name the documents that answer it"
    );
}
