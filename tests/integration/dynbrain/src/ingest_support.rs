//! Deterministic stand-ins for the embedding and synopsis models, and the
//! steps the indexing suites share. Nothing here reaches a real provider.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use dynbrain::BrainRepository;
use dynbrain::ingest::context::SynopsisPrompt;
use dynbrain::ingest::index::Indexer;
use dynbrain::ingest::provider::{DIMENSIONS, Embedder, Embedding, Failure, Summarizer};
use dynbrain::ingest::run::{IndexSettings, RunReport};

use crate::fixture::Fixture;

pub(crate) struct FakeEmbedder {
    pub(crate) name: String,
    pub(crate) failure: Mutex<Option<Failure>>,
    pub(crate) texts: Mutex<Vec<String>>,
    pub(crate) requests: Mutex<usize>,
}

impl FakeEmbedder {
    pub(crate) fn named(name: &str) -> Arc<Self> {
        Arc::new(Self {
            name: name.to_owned(),
            failure: Mutex::new(None),
            texts: Mutex::new(Vec::new()),
            requests: Mutex::new(0),
        })
    }

    pub(crate) fn fail_with(&self, failure: Option<Failure>) {
        *self.failure.lock().expect("the failure") = failure;
    }

    pub(crate) fn requests(&self) -> usize {
        *self.requests.lock().expect("the request count")
    }

    pub(crate) fn seen(&self) -> Vec<String> {
        self.texts.lock().expect("the texts").clone()
    }
}

#[async_trait]
impl Embedder for FakeEmbedder {
    fn fingerprint(&self) -> String {
        format!("{}:{DIMENSIONS}", self.name)
    }

    async fn embed(&self, texts: &[String]) -> Result<Embedding, Failure> {
        *self.requests.lock().expect("the request count") += 1;
        if let Some(failure) = self.failure.lock().expect("the failure").clone() {
            return Err(failure);
        }
        self.texts
            .lock()
            .expect("the texts")
            .extend_from_slice(texts);
        Ok(Embedding {
            vectors: texts
                .iter()
                .map(|text| vec![0.1 + (text.len() % 7) as f32 / 10.0; DIMENSIONS])
                .collect(),
            input_tokens: 10,
            cost_microdollars: 3,
        })
    }
}

#[derive(Default)]
pub(crate) struct FakeSummarizer {
    pub(crate) prompts: Mutex<Vec<SynopsisPrompt>>,
}

impl FakeSummarizer {
    pub(crate) fn shared() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub(crate) fn prompts(&self) -> Vec<SynopsisPrompt> {
        self.prompts.lock().expect("the prompts").clone()
    }
}

#[async_trait]
impl Summarizer for FakeSummarizer {
    async fn summarize(&self, prompt: &SynopsisPrompt) -> Result<String, Failure> {
        self.prompts
            .lock()
            .expect("the prompts")
            .push(prompt.clone());
        let excerpt = prompt.user.rsplit("<excerpt>\n").next().unwrap_or_default();
        let lead: Vec<&str> = excerpt.split_whitespace().take(3).collect();
        Ok(format!("About {}", lead.join(" ")))
    }
}

pub(crate) fn indexer(
    repo: &BrainRepository,
    embedder: &Arc<FakeEmbedder>,
    summarizer: &Arc<FakeSummarizer>,
) -> Indexer {
    let embedder: Arc<dyn Embedder> = Arc::<FakeEmbedder>::clone(embedder);
    let summarizer: Arc<dyn Summarizer> = Arc::<FakeSummarizer>::clone(summarizer);
    Indexer::new(
        repo.clone(),
        embedder,
        Some(summarizer),
        IndexSettings {
            synopsis_backoff: Duration::ZERO,
            ..IndexSettings::default()
        },
    )
}

pub(crate) async fn include_everything(f: &Fixture) {
    sqlx::query(
        "INSERT INTO dynbrain_rag_rule (name, scope, mode) VALUES ('everything', 'all', 'include')",
    )
    .execute(f.pool())
    .await
    .expect("add a rule");
}

pub(crate) async fn count(f: &Fixture, sql: &str) -> i64 {
    sqlx::query_scalar(sql)
        .fetch_one(f.pool())
        .await
        .expect("count")
}

pub(crate) async fn run(indexer: &Indexer) -> RunReport {
    indexer.run().await.expect("an indexing run")
}

pub(crate) fn plain_body(sentences: usize) -> String {
    (0..sentences)
        .map(|n| format!("w{n}a w{n}b w{n}c w{n}d w{n}e w{n}f w{n}g w{n}h w{n}i end{n}."))
        .collect::<Vec<_>>()
        .join(" ")
}
