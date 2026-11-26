//! The rows behind the results: the fused pool, a document's chunks ranked
//! for the answer context, the fingerprint check of the vector arm, and the
//! query-vector cache.
//!
//! The pool is read by chunk id after the arms have applied the reader's
//! access, so it needs no access check of its own. The cache is the engine's:
//! it holds what people searched for and is read by nobody else.

use super::BrainRepository;
use crate::error::BrainError;
use crate::ids::{ArticleId, ChunkId, SpaceId};
use crate::search::context::ContextChunk;
use crate::search::query::{QueryKey, QueryVector};
use crate::search::service::Passage;

struct PassageRow {
    id: i64,
    article_id: i64,
    space_id: i64,
    lang: String,
    title: String,
    description: String,
    content: String,
}

struct ContextRow {
    id: i64,
    sequence: i32,
    content: String,
}

impl BrainRepository {
    pub async fn list_passages(&self, chunks: &[ChunkId]) -> Result<Vec<Passage>, BrainError> {
        let ids: Vec<i64> = chunks.iter().map(|chunk| chunk.0).collect();
        let rows = sqlx::query_as!(
            PassageRow,
            r#"SELECT c.id AS "id!", c.article_id AS "article_id!", a.space_id AS "space_id!",
                      c.lang AS "lang!", v.title AS "title!", v.description AS "description!",
                      c.content AS "content!"
                 FROM dynbrain_chunk c
                 JOIN dynbrain_article a ON a.id = c.article_id
                 JOIN dynbrain_article_variant v ON v.article_id = c.article_id AND v.lang = c.lang
                WHERE c.id = ANY($1::int8[])"#,
            ids.as_slice()
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| Passage {
                chunk: Some(ChunkId(row.id)),
                article: ArticleId(row.article_id),
                space: SpaceId(row.space_id),
                lang: row.lang,
                title: row.title,
                description: row.description,
                content: row.content,
                score: None,
            })
            .collect())
    }

    pub async fn list_context_chunks(
        &self,
        article: ArticleId,
        lang: &str,
        vector: &QueryVector,
    ) -> Result<Vec<ContextChunk>, BrainError> {
        let rows = sqlx::query_as!(
            ContextRow,
            r#"SELECT id AS "id!", sequence AS "sequence!", content AS "content!"
                 FROM dynbrain_chunk
                WHERE article_id = $1 AND lang = $2 AND embedding IS NOT NULL
                ORDER BY embedding <=> $3::text::vector, id"#,
            article.0,
            lang,
            vector.0.as_str()
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| ContextChunk {
                id: ChunkId(row.id),
                sequence: row.sequence,
                content: row.content,
            })
            .collect())
    }

    pub async fn get_foreign_vectors(&self, fingerprint: &str) -> Result<bool, BrainError> {
        let foreign = sqlx::query_scalar!(
            r#"SELECT EXISTS (SELECT 1 FROM dynbrain_chunk
                               WHERE embedding IS NOT NULL AND embedding_fingerprint IS NOT NULL
                                 AND embedding_fingerprint <> $1) AS "foreign!""#,
            fingerprint
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(foreign)
    }

    pub async fn find_cached_query(&self, hash: &str) -> Result<Option<String>, BrainError> {
        let vector = sqlx::query_scalar!(
            r#"UPDATE dynbrain_query_cache SET hits = hits + 1, last_used = CURRENT_TIMESTAMP
                WHERE query_hash = $1
            RETURNING embedding::text AS "embedding!""#,
            hash
        )
        .fetch_optional(&*self.pool)
        .await?;
        Ok(vector)
    }

    pub async fn insert_cached_query(
        &self,
        key: &QueryKey,
        query: &str,
        vector: &QueryVector,
    ) -> Result<(), BrainError> {
        sqlx::query!(
            r#"INSERT INTO dynbrain_query_cache (query_hash, query_text, provider, model, embedding, hits)
               VALUES ($1, $2, $3, $4, $5::text::vector, 1)
               ON CONFLICT (query_hash) DO UPDATE
                  SET hits = dynbrain_query_cache.hits + 1, last_used = CURRENT_TIMESTAMP"#,
            key.hash,
            query,
            key.provider,
            key.model,
            vector.0.as_str()
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_stale_queries(&self, days: i32) -> Result<u64, BrainError> {
        let done = sqlx::query!(
            "DELETE FROM dynbrain_query_cache WHERE last_used < CURRENT_TIMESTAMP - make_interval(days => $1)",
            days
        )
        .execute(&*self.pool)
        .await?;
        Ok(done.rows_affected())
    }
}
