//! What the admin's Brain section reads and does beyond the content: the
//! chunks of a document, the indexing queue and the counters. The usage log
//! and the audit are in [`super::usage`].

use chrono::{DateTime, Utc};

use super::BrainRepository;
use crate::error::BrainError;
use crate::ids::ArticleId;
use crate::search::query::QueryVector;

/// A chunk's scores in the two arms: cosine similarity and `ts_rank`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArmRow {
    pub id: i64,
    pub cosine: Option<f64>,
    pub ts_rank: f64,
}

/// A chunk as the index holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkInfo {
    pub id: i64,
    pub sequence: i32,
    pub heading_path: String,
    pub content: String,
    pub synopsis: Option<String>,
    pub fts_config: String,
    pub embedded: bool,
    pub fingerprint: Option<String>,
    pub has_error: bool,
    pub synopsis_error: bool,
    pub embedded_at: Option<DateTime<Utc>>,
}

/// One indexing job, with the title of the variant it indexes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobInfo {
    pub id: i64,
    pub article_id: i64,
    pub lang: String,
    pub title: Option<String>,
    pub content_hash: String,
    pub state: String,
    pub attempts: i32,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// The index and its queue, counted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexCounts {
    pub articles: i64,
    pub documents: i64,
    pub chunks: i64,
    pub pending_embeddings: i64,
    pub pending_synopses: i64,
    pub chunk_errors: i64,
    pub queued_jobs: i64,
    pub failed_jobs: i64,
}

impl BrainRepository {
    pub async fn list_arm_scores(
        &self,
        chunks: &[i64],
        vector: &QueryVector,
        query: &str,
    ) -> Result<Vec<ArmRow>, BrainError> {
        Ok(sqlx::query_as!(
            ArmRow,
            r#"SELECT c.id AS "id!",
                      (1 - (c.embedding <=> $2::text::vector))::float8 AS "cosine?",
                      ts_rank(c.search_vector, plainto_tsquery(c.fts_config, $3::text))::float8 AS "ts_rank!"
                 FROM dynbrain_chunk c
                WHERE c.id = ANY($1)"#,
            chunks,
            vector.0.as_str(),
            query
        )
        .fetch_all(&*self.pool)
        .await?)
    }

    pub async fn list_chunk_infos(
        &self,
        article: ArticleId,
        lang: &str,
    ) -> Result<Vec<ChunkInfo>, BrainError> {
        Ok(sqlx::query_as!(
            ChunkInfo,
            r#"SELECT c.id AS "id!", c.sequence, c.heading_path, c.content, c.synopsis,
                      c.fts_config::text AS "fts_config!",
                      (c.embedding IS NOT NULL) AS "embedded!",
                      c.embedding_fingerprint AS fingerprint,
                      c.has_error, c.synopsis_error, c.embedded_at
                 FROM dynbrain_chunk c
                WHERE c.article_id = $1 AND c.lang = $2
                ORDER BY c.sequence"#,
            article.0,
            lang
        )
        .fetch_all(&*self.pool)
        .await?)
    }

    pub async fn list_index_jobs(
        &self,
        state: Option<&str>,
        limit: i64,
    ) -> Result<Vec<JobInfo>, BrainError> {
        Ok(sqlx::query_as!(
            JobInfo,
            r#"SELECT j.id AS "id!", j.article_id, j.lang, v.title AS "title?", j.content_hash,
                      j.state, j.attempts, j.last_error, j.created_at, j.updated_at
                 FROM dynbrain_index_job j
                 LEFT JOIN dynbrain_article_variant v ON v.article_id = j.article_id AND v.lang = j.lang
                WHERE ($1::text IS NULL OR j.state = $1)
                ORDER BY (j.state = 'failed') DESC, j.updated_at DESC, j.id DESC
                LIMIT $2"#,
            state,
            limit
        )
        .fetch_all(&*self.pool)
        .await?)
    }

    pub async fn list_document_jobs(
        &self,
        article: ArticleId,
        lang: &str,
    ) -> Result<Vec<JobInfo>, BrainError> {
        Ok(sqlx::query_as!(
            JobInfo,
            r#"SELECT j.id AS "id!", j.article_id, j.lang, v.title AS "title?", j.content_hash,
                      j.state, j.attempts, j.last_error, j.created_at, j.updated_at
                 FROM dynbrain_index_job j
                 LEFT JOIN dynbrain_article_variant v ON v.article_id = j.article_id AND v.lang = j.lang
                WHERE j.article_id = $1 AND j.lang = $2
                ORDER BY j.updated_at DESC, j.id DESC
                LIMIT 10"#,
            article.0,
            lang
        )
        .fetch_all(&*self.pool)
        .await?)
    }

    pub async fn requeue_variant(&self, article: ArticleId, lang: &str) -> Result<u64, BrainError> {
        Ok(sqlx::query!(
            "INSERT INTO dynbrain_index_job (article_id, lang, content_hash)
             SELECT v.article_id, v.lang, v.content_hash
               FROM dynbrain_article_variant v
              WHERE v.article_id = $1 AND v.lang = $2
             ON CONFLICT (article_id, lang, content_hash)
             DO UPDATE SET state = 'pending', attempts = 0, last_error = NULL,
                           updated_at = CURRENT_TIMESTAMP",
            article.0,
            lang
        )
        .execute(&*self.pool)
        .await?
        .rows_affected())
    }

    pub async fn retry_job(&self, job: i64) -> Result<u64, BrainError> {
        Ok(sqlx::query!(
            "UPDATE dynbrain_index_job
                SET state = 'pending', attempts = 0, last_error = NULL,
                    updated_at = CURRENT_TIMESTAMP
              WHERE id = $1 AND state = 'failed'",
            job
        )
        .execute(&*self.pool)
        .await?
        .rows_affected())
    }

    pub async fn count_index(&self) -> Result<IndexCounts, BrainError> {
        Ok(sqlx::query_as!(
            IndexCounts,
            r#"SELECT
                 (SELECT count(*) FROM dynbrain_article WHERE trashed_at IS NULL) AS "articles!",
                 (SELECT count(*) FROM (SELECT DISTINCT article_id, lang FROM dynbrain_chunk) d) AS "documents!",
                 (SELECT count(*) FROM dynbrain_chunk) AS "chunks!",
                 (SELECT count(*) FROM dynbrain_chunk WHERE embedding IS NULL AND NOT has_error) AS "pending_embeddings!",
                 (SELECT count(*) FROM dynbrain_chunk
                   WHERE heading_path = '' AND synopsis IS NULL AND NOT synopsis_error) AS "pending_synopses!",
                 (SELECT count(*) FROM dynbrain_chunk WHERE has_error OR synopsis_error) AS "chunk_errors!",
                 (SELECT count(*) FROM dynbrain_index_job WHERE state IN ('pending', 'running')) AS "queued_jobs!",
                 (SELECT count(*) FROM dynbrain_index_job WHERE state = 'failed') AS "failed_jobs!""#
        )
        .fetch_one(&*self.pool)
        .await?)
    }
}
