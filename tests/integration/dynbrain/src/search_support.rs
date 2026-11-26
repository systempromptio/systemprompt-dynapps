//! A corpus written straight into the index, and doubles of the models.
//!
//! The query embedder and the reranker are deterministic stand-ins; nothing
//! here reaches a provider.
//!
//! Vectors are one-hot over the 1536 dimensions, so cosine distance orders
//! chunks exactly: a chunk on the query's axis is nearest.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use dynbrain::access::{Level, Subject};
use dynbrain::brain::args::SearchArgs;
use dynbrain::brain::native::NativeBrain;
use dynbrain::ids::{ChunkId, SpaceId};
use dynbrain::ingest::provider::{DIMENSIONS, Embedder, Embedding, Failure};
use dynbrain::model::{Article, NewArticle, RagMode, VariantWrite};
use dynbrain::search::query::EmbedderSlot;
use dynbrain::search::rerank::{RerankFailure, Reranker};
use dynbrain::search::{RetrievalSettings, Retriever};
use serde_json::{Map, Value};

use crate::fixture::{Fixture, user};

pub(crate) const FINGERPRINT: &str = "test:query:1536";

pub(crate) fn axis(hot: &[usize]) -> Vec<f32> {
    let mut vector = vec![0.0; DIMENSIONS];
    for dimension in hot {
        vector[*dimension] = 1.0;
    }
    vector
}

fn as_text(vector: &[f32]) -> String {
    let parts: Vec<String> = vector.iter().map(ToString::to_string).collect();
    format!("[{}]", parts.join(","))
}

// Embeds every query on the same axis and counts the calls.
pub(crate) struct QueryEmbedder {
    vector: Vec<f32>,
    calls: AtomicUsize,
}

impl QueryEmbedder {
    pub(crate) fn on(hot: &[usize]) -> Arc<Self> {
        Arc::new(Self {
            vector: axis(hot),
            calls: AtomicUsize::new(0),
        })
    }

    pub(crate) fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl Embedder for QueryEmbedder {
    fn fingerprint(&self) -> String {
        FINGERPRINT.to_owned()
    }

    async fn embed(&self, texts: &[String]) -> Result<Embedding, Failure> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(Embedding {
            vectors: texts.iter().map(|_| self.vector.clone()).collect(),
            input_tokens: 1,
            cost_microdollars: 0,
        })
    }
}

// Scores each document by the first marker it contains, or fails.
#[derive(Debug)]
pub(crate) struct ScriptedReranker {
    scores: Vec<(&'static str, f64)>,
    failure: Option<u16>,
    calls: AtomicUsize,
}

impl ScriptedReranker {
    pub(crate) fn scoring(scores: &[(&'static str, f64)]) -> Arc<Self> {
        Arc::new(Self {
            scores: scores.to_vec(),
            failure: None,
            calls: AtomicUsize::new(0),
        })
    }

    pub(crate) fn failing(status: u16) -> Arc<Self> {
        Arc::new(Self {
            scores: Vec::new(),
            failure: Some(status),
            calls: AtomicUsize::new(0),
        })
    }

    pub(crate) fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl Reranker for ScriptedReranker {
    fn provider(&self) -> &'static str {
        "scripted"
    }

    fn endpoint(&self) -> &'static str {
        "test://rerank"
    }

    async fn rerank(
        &self,
        _query: &str,
        documents: &[String],
    ) -> Result<Vec<(usize, f64)>, RerankFailure> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if let Some(status) = self.failure {
            return Err(RerankFailure::Status(status));
        }
        let mut ranked: Vec<(usize, f64)> = documents
            .iter()
            .enumerate()
            .map(|(index, text)| {
                let score = self
                    .scores
                    .iter()
                    .find(|(marker, _)| text.contains(marker))
                    .map_or(0.05, |(_, score)| *score);
                (index, score)
            })
            .collect();
        ranked.sort_by(|left, right| right.1.total_cmp(&left.1));
        Ok(ranked)
    }
}

pub(crate) fn retriever(
    f: &Fixture,
    embedder: Option<Arc<QueryEmbedder>>,
    reranker: Option<Arc<ScriptedReranker>>,
    settings: RetrievalSettings,
) -> Retriever {
    let mut built = Retriever::new(f.repo.clone()).with_settings(settings);
    if let Some(embedder) = embedder {
        built = built.with_embedder(EmbedderSlot::ready(embedder));
    }
    if let Some(reranker) = reranker {
        built = built.with_reranker(Some(reranker));
    }
    built
}

pub(crate) fn brain(f: &Fixture, retriever: Retriever) -> NativeBrain {
    NativeBrain::new(f.repo.clone()).with_search(retriever)
}

pub(crate) fn hybrid(f: &Fixture) -> NativeBrain {
    brain(
        f,
        retriever(
            f,
            Some(QueryEmbedder::on(&[0])),
            None,
            RetrievalSettings::default(),
        ),
    )
}

pub(crate) fn ask(query: &str) -> SearchArgs {
    SearchArgs {
        query: query.to_owned(),
        ..SearchArgs::default()
    }
}

// An article of `space` with one variant per (lang, title, body).
pub(crate) async fn doc(
    f: &Fixture,
    space: SpaceId,
    slug: &str,
    variants: &[(&str, &str, &str)],
) -> Article {
    let article = f
        .repo
        .insert_article(
            &NewArticle {
                space,
                parent: None,
                position: None,
                slug: slug.to_owned(),
                icon: None,
                rag_mode: RagMode::Default,
            },
            &user("alice"),
        )
        .await
        .expect("create an article");
    for (lang, title, body) in variants {
        f.write(&article, lang, title, body).await;
    }
    article
}

pub(crate) async fn with_properties(f: &Fixture, article: &Article, properties: &Value) {
    let write = VariantWrite {
        title: format!("Doc {}", article.slug),
        description: String::new(),
        body: format!("Body of {}.", article.slug),
        front_matter: Map::new(),
        properties: properties.as_object().cloned().expect("an object"),
    };
    f.repo
        .write_variant(article.id, "es", &write, &user("alice"))
        .await
        .expect("write a variant");
}

// A chunk written straight into the index, embedded with the suite's
// fingerprint when it has a vector.
pub(crate) async fn chunk(
    f: &Fixture,
    article: &Article,
    at: (&str, i32),
    content: &str,
    vector: Option<&[f32]>,
) -> ChunkId {
    chunk_by(
        f,
        article,
        at,
        content,
        vector.map(|vector| (vector, FINGERPRINT)),
    )
    .await
}

// A chunk embedded by the configuration `fingerprint` names.
pub(crate) async fn chunk_by(
    f: &Fixture,
    article: &Article,
    (lang, sequence): (&str, i32),
    content: &str,
    embedded: Option<(&[f32], &str)>,
) -> ChunkId {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO dynbrain_chunk
                (article_id, lang, sequence, content, content_hash, fts_config,
                 embedding, embedding_fingerprint, embedded_at)
         VALUES ($1, $2, $3, $4, md5($4), 'simple'::regconfig, $5::text::vector,
                 CASE WHEN $5::text IS NULL THEN NULL ELSE $6 END,
                 CASE WHEN $5::text IS NULL THEN NULL ELSE CURRENT_TIMESTAMP END)
         RETURNING id",
    )
    .bind(article.id.0)
    .bind(lang)
    .bind(sequence)
    .bind(content)
    .bind(embedded.map(|(vector, _)| as_text(vector)))
    .bind(embedded.map(|(_, fingerprint)| fingerprint))
    .fetch_one(f.pool())
    .await
    .expect("index a chunk");
    ChunkId(id)
}

pub(crate) async fn let_read(f: &Fixture, article: &Article, who: &str) {
    f.give(
        article.space,
        Some(article.id),
        Subject::user(&user(who)),
        Level::Read,
    )
    .await;
}
