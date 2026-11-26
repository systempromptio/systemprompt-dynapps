//! Embedding a batch that a provider refuses in part, or all of.

use std::sync::Mutex;

use async_trait::async_trait;
use dynbrain::ingest::bisect::{Candidate, EmbeddingRun, embed_bisecting};
use dynbrain::ingest::provider::{DIMENSIONS, Embedder, Embedding, Failure};

#[derive(Default)]
struct Scripted {
    bad: Vec<String>,
    refuse_all: bool,
    limited: bool,
    short_vectors: bool,
    calls: Mutex<Vec<usize>>,
}

impl Scripted {
    fn calls(&self) -> Vec<usize> {
        self.calls.lock().expect("the call log").clone()
    }
}

#[async_trait]
impl Embedder for Scripted {
    fn fingerprint(&self) -> String {
        format!("test:model:{DIMENSIONS}")
    }

    async fn embed(&self, texts: &[String]) -> Result<Embedding, Failure> {
        self.calls.lock().expect("the call log").push(texts.len());
        if self.limited {
            return Err(Failure::RateLimited);
        }
        if self.refuse_all || texts.iter().any(|text| self.bad.contains(text)) {
            return Err(Failure::Rejected("refused".to_owned()));
        }
        let size = if self.short_vectors { 3 } else { DIMENSIONS };
        Ok(Embedding {
            vectors: texts.iter().map(|_| vec![0.5; size]).collect(),
            input_tokens: texts.len() as u32,
            cost_microdollars: 7,
        })
    }
}

fn candidates(count: i64) -> Vec<Candidate> {
    (0..count)
        .map(|id| Candidate {
            id,
            text: format!("text {id}"),
            context_hash: format!("hash {id}"),
        })
        .collect()
}

async fn run(embedder: &Scripted, count: i64) -> EmbeddingRun {
    embed_bisecting(embedder, &candidates(count)).await
}

#[tokio::test]
async fn a_batch_the_provider_accepts_is_one_request() {
    let embedder = Scripted::default();
    let done = run(&embedder, 50).await;
    assert_eq!(embedder.calls(), [50]);
    assert_eq!(done.vectors.len(), 50);
    assert!(done.rejected.is_empty() && done.stopped.is_none());
    assert_eq!(done.spends.len(), 1);
    assert_eq!(done.spends[0].cost_microdollars, 7);
    assert_eq!(done.vectors[3].id, 3);
    assert_eq!(done.vectors[3].context_hash, "hash 3");
}

#[tokio::test]
async fn one_refused_text_is_found_in_a_few_requests_and_the_rest_is_embedded() {
    let embedder = Scripted {
        bad: vec!["text 17".to_owned()],
        ..Scripted::default()
    };
    let done = run(&embedder, 50).await;
    assert_eq!(done.rejected, [17]);
    assert_eq!(done.vectors.len(), 49);
    assert!(done.vectors.iter().all(|vector| vector.id != 17));
    let calls = embedder.calls();
    assert!(calls.len() <= 14, "{calls:?}");
    assert!(done.stopped.is_none());
}

#[tokio::test]
async fn a_provider_that_refuses_everything_costs_three_requests_and_marks_nobody() {
    let embedder = Scripted {
        refuse_all: true,
        ..Scripted::default()
    };
    let done = run(&embedder, 50).await;
    assert_eq!(embedder.calls(), [50, 25, 25]);
    assert!(done.rejected.is_empty() && done.vectors.is_empty() && done.stopped.is_none());
}

#[tokio::test]
async fn vectors_of_the_wrong_size_are_a_refusal_of_the_batch() {
    let embedder = Scripted {
        short_vectors: true,
        ..Scripted::default()
    };
    let done = run(&embedder, 10).await;
    assert!(done.vectors.is_empty() && done.rejected.is_empty());
    assert!(done.spends.is_empty());
}

#[tokio::test]
async fn a_rate_limit_stops_the_run_and_marks_nobody() {
    let embedder = Scripted {
        limited: true,
        ..Scripted::default()
    };
    let done = run(&embedder, 50).await;
    assert_eq!(embedder.calls(), [50]);
    assert_eq!(done.stopped, Some(Failure::RateLimited));
    assert!(done.rejected.is_empty() && done.vectors.is_empty());
}

#[tokio::test]
async fn a_lone_text_that_is_refused_is_marked() {
    let embedder = Scripted {
        refuse_all: true,
        ..Scripted::default()
    };
    let done = run(&embedder, 1).await;
    assert_eq!(done.rejected, [0]);
    assert_eq!(embedder.calls(), [1]);
}
