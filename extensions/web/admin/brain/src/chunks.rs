//! The Chunks and Ingest tabs of an article: how the index holds one
//! variant and where it is in the queue.

use dynbrain::console::Native;
use dynbrain::ids::ArticleId;
use dynbrain::ingest::context::{ChunkContext, ContextStrategy, document_context, is_code_chunk, prefix};
use dynbrain::repository::admin::{ChunkInfo, JobInfo};
use serde::Serialize;
use systemprompt_web_admin_core::ssr::faults::Faults;

use crate::read::Document;
use crate::views::{keep, state_tone, when, words};

#[derive(Debug, Serialize)]
pub struct ChunkRowView {
    pub n: i32,
    pub path: String,
    pub words: usize,
    pub fts: String,
    pub code: bool,
    pub embedding: &'static str,
    pub tone: &'static str,
    pub fingerprint: String,
    pub embedded_at: String,
    pub prefix: String,
    pub synopsis: String,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct ChunksView {
    pub context: String,
    pub rows: Vec<ChunkRowView>,
    pub has_rows: bool,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct StepView {
    pub name: &'static str,
    pub state: &'static str,
    pub tone: &'static str,
}

#[derive(Debug, Serialize)]
pub struct JobRowView {
    pub id: i64,
    pub when: String,
    pub state: String,
    pub tone: &'static str,
    pub attempts: i32,
    pub error: String,
    pub retryable: bool,
}

#[derive(Debug, Serialize)]
pub struct IngestView {
    pub steps: Vec<StepView>,
    pub jobs: Vec<JobRowView>,
    pub has_jobs: bool,
    pub article: i64,
    pub lang: String,
    pub chunks: usize,
}

pub async fn chunks_view(
    native: &Native,
    article: i64,
    doc: &Document,
    faults: &mut Faults,
) -> Option<ChunksView> {
    let chunks = keep(
        faults,
        "Chunks",
        native.repo.list_chunk_infos(ArticleId(article), &doc.lang).await,
    )?;
    let context = document_context(&doc.title, &doc.description);
    let rows: Vec<ChunkRowView> = chunks.iter().map(|chunk| row(chunk, &context)).collect();
    Some(ChunksView {
        context,
        has_rows: !rows.is_empty(),
        rows,
    })
}

fn row(chunk: &ChunkInfo, context: &str) -> ChunkRowView {
    let (embedding, tone) = embedding_state(chunk);
    let code = is_code_chunk(&chunk.content);
    let parts = ChunkContext {
        content: &chunk.content,
        heading_path: &chunk.heading_path,
        synopsis: chunk.synopsis.as_deref(),
    };
    ChunkRowView {
        n: chunk.sequence,
        path: chunk.heading_path.clone(),
        words: words(&chunk.content),
        fts: chunk.fts_config.clone(),
        code,
        embedding,
        tone,
        fingerprint: chunk.fingerprint.clone().unwrap_or_default(),
        embedded_at: chunk.embedded_at.map(when).unwrap_or_default(),
        prefix: prefix(ContextStrategy::Auto, context, &parts),
        synopsis: chunk.synopsis.clone().unwrap_or_default(),
        text: chunk.content.clone(),
    }
}

fn embedding_state(chunk: &ChunkInfo) -> (&'static str, &'static str) {
    if chunk.has_error {
        ("error", "err")
    } else if chunk.embedded {
        ("ok", "ok")
    } else {
        ("pending", "warn")
    }
}

pub async fn ingest_view(
    native: &Native,
    article: i64,
    lang: &str,
    faults: &mut Faults,
) -> Option<IngestView> {
    let chunks = keep(
        faults,
        "Chunks",
        native.repo.list_chunk_infos(ArticleId(article), lang).await,
    )?;
    let jobs = keep(
        faults,
        "Ingest",
        native.repo.list_document_jobs(ArticleId(article), lang).await,
    )?;
    let steps = steps(&chunks, jobs.first());
    let rows: Vec<JobRowView> = jobs.iter().map(job_row).collect();
    Some(IngestView {
        steps,
        has_jobs: !rows.is_empty(),
        jobs: rows,
        article,
        lang: lang.to_owned(),
        chunks: chunks.len(),
    })
}

fn job_row(job: &JobInfo) -> JobRowView {
    JobRowView {
        id: job.id,
        when: when(job.updated_at),
        tone: state_tone(&job.state),
        retryable: job.state == "failed",
        state: job.state.clone(),
        attempts: job.attempts,
        error: job.last_error.clone().unwrap_or_default(),
    }
}

fn steps(chunks: &[ChunkInfo], job: Option<&JobInfo>) -> Vec<StepView> {
    let failed = job.is_some_and(|job| job.state == "failed");
    let any = !chunks.is_empty();
    let chunked = if any {
        "ok"
    } else if failed {
        "error"
    } else {
        "todo"
    };
    let synopsis = if !any {
        "todo"
    } else if chunks.iter().any(|chunk| chunk.synopsis_error) {
        "error"
    } else if chunks
        .iter()
        .any(|chunk| chunk.heading_path.is_empty() && chunk.synopsis.is_none())
    {
        "pending"
    } else {
        "ok"
    };
    let embeddings = if !any {
        "todo"
    } else if chunks.iter().any(|chunk| chunk.has_error) {
        "error"
    } else if chunks.iter().any(|chunk| !chunk.embedded) {
        "pending"
    } else {
        "ok"
    };
    let settled = job.is_none_or(|job| job.state == "done");
    let searchable = if any && embeddings == "ok" && settled {
        "ok"
    } else {
        "todo"
    };
    [
        ("Saved", "ok"),
        ("Chunked", chunked),
        ("Synopsis", synopsis),
        ("Embeddings", embeddings),
        ("Searchable", searchable),
    ]
    .into_iter()
    .map(|(name, state)| StepView {
        name,
        state,
        tone: state_tone(state),
    })
    .collect()
}
