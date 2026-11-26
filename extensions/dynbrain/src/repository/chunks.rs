//! The chunks of the retrieval index and their embeddings.
//!
//! A chunk is rebuilt whenever its variant is indexed again: the old ones are
//! deleted and the new ones inserted in one transaction, and a chunk whose
//! text did not change keeps the synopsis it had. A synopsis is bound to the
//! text it describes (`synopsis_hash`), so a changed text cannot keep a
//! description of something else. A vector is bound to the configuration
//! that made it (`embedding_fingerprint`) and to the prefix it was embedded
//! with (`context_hash`); a chunk with no `embedded_at` is queued for one.
//! Vectors cross as text (`[0.1,0.2,...]`) and are cast by Postgres.

use std::collections::HashMap;

use super::BrainRepository;
use crate::error::BrainError;
use crate::ids::ArticleId;
use crate::ingest::bisect::{Spend, Vectored};
use crate::ingest::model::{EmbedTask, FingerprintCount, NewChunk, SynopsisTask, SynopsisWrite};

struct CarriedRow {
    content_hash: String,
    synopsis: String,
}

struct SynopsisRow {
    id: i64,
    lang: String,
    content: String,
    content_hash: String,
    body: String,
}

struct EmbedRow {
    id: i64,
    content: String,
    heading_path: String,
    synopsis: Option<String>,
    title: String,
    description: String,
}

struct FingerprintRow {
    fingerprint: String,
    chunks: i64,
}

impl BrainRepository {
    pub async fn carried_synopses(
        &self,
        article: ArticleId,
        lang: &str,
    ) -> Result<HashMap<String, String>, BrainError> {
        let rows = sqlx::query_as!(
            CarriedRow,
            r#"SELECT content_hash, synopsis AS "synopsis!"
                 FROM dynbrain_chunk
                WHERE article_id = $1 AND lang = $2
                  AND synopsis IS NOT NULL AND synopsis_hash = content_hash"#,
            article.0,
            lang
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| (row.content_hash, row.synopsis))
            .collect())
    }

    pub async fn replace_chunks(
        &self,
        article: ArticleId,
        lang: &str,
        fts_config: &str,
        chunks: &[NewChunk],
    ) -> Result<(), BrainError> {
        let sequences: Vec<i32> = (0..chunks.len()).map(|at| at as i32).collect();
        let paths: Vec<String> = chunks.iter().map(|c| c.heading_path.clone()).collect();
        let contents: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let hashes: Vec<String> = chunks.iter().map(|c| c.content_hash.clone()).collect();
        let synopses: Vec<String> = chunks
            .iter()
            .map(|c| c.synopsis.clone().unwrap_or_default())
            .collect();
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            "DELETE FROM dynbrain_chunk WHERE article_id = $1 AND lang = $2",
            article.0,
            lang
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query!(
            "INSERT INTO dynbrain_chunk
                    (article_id, lang, sequence, heading_path, content, content_hash,
                     synopsis, synopsis_hash, fts_config)
             SELECT $1::int8, $2::text, u.sequence, u.heading_path, u.content, u.content_hash,
                    NULLIF(u.synopsis, ''),
                    CASE WHEN u.synopsis = '' THEN NULL ELSE u.content_hash END,
                    $3::text::regconfig
               FROM UNNEST($4::int4[], $5::text[], $6::text[], $7::text[], $8::text[])
                    AS u(sequence, heading_path, content, content_hash, synopsis)",
            article.0,
            lang,
            fts_config,
            sequences.as_slice(),
            paths.as_slice(),
            contents.as_slice(),
            hashes.as_slice(),
            synopses.as_slice()
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn purge_chunks(&self, article: ArticleId, lang: &str) -> Result<u64, BrainError> {
        let done = sqlx::query!(
            "DELETE FROM dynbrain_chunk WHERE article_id = $1 AND lang = $2",
            article.0,
            lang
        )
        .execute(&*self.pool)
        .await?;
        Ok(done.rows_affected())
    }

    pub async fn synopsis_tasks(
        &self,
        limit: i64,
        with_headings: bool,
    ) -> Result<Vec<SynopsisTask>, BrainError> {
        let rows = sqlx::query_as!(
            SynopsisRow,
            "SELECT c.id, c.lang, c.content, c.content_hash, v.body
               FROM dynbrain_chunk c
               JOIN dynbrain_article_variant v ON v.article_id = c.article_id AND v.lang = c.lang
               JOIN dynbrain_article a ON a.id = c.article_id
              WHERE c.synopsis_hash IS DISTINCT FROM c.content_hash
                AND NOT c.synopsis_error AND a.trashed_at IS NULL
                AND ($2::bool OR c.heading_path = '')
              ORDER BY c.article_id, c.lang, c.sequence
              LIMIT $1",
            limit,
            with_headings
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| SynopsisTask {
                id: row.id,
                lang: row.lang,
                content: row.content,
                content_hash: row.content_hash,
                body: row.body,
            })
            .collect())
    }

    pub async fn store_synopses(&self, writes: &[SynopsisWrite]) -> Result<u64, BrainError> {
        let ids: Vec<i64> = writes.iter().map(|w| w.id).collect();
        let texts: Vec<String> = writes.iter().map(|w| w.text.clone()).collect();
        let hashes: Vec<String> = writes.iter().map(|w| w.content_hash.clone()).collect();
        let done = sqlx::query!(
            "UPDATE dynbrain_chunk c
                SET synopsis = u.synopsis, synopsis_hash = u.content_hash, embedded_at = NULL
               FROM UNNEST($1::int8[], $2::text[], $3::text[]) AS u(id, synopsis, content_hash)
              WHERE c.id = u.id AND c.content_hash = u.content_hash",
            ids.as_slice(),
            texts.as_slice(),
            hashes.as_slice()
        )
        .execute(&*self.pool)
        .await?;
        Ok(done.rows_affected())
    }

    pub async fn mark_synopsis_errors(&self, ids: &[i64]) -> Result<(), BrainError> {
        sqlx::query!(
            "UPDATE dynbrain_chunk SET synopsis_error = true WHERE id = ANY($1::int8[])",
            ids
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    pub async fn embed_tasks(
        &self,
        limit: i64,
        wait_for_all: bool,
        wait_for_headingless: bool,
    ) -> Result<Vec<EmbedTask>, BrainError> {
        let rows = sqlx::query_as!(
            EmbedRow,
            "SELECT c.id, c.content, c.heading_path, c.synopsis, v.title, v.description
               FROM dynbrain_chunk c
               JOIN dynbrain_article_variant v ON v.article_id = c.article_id AND v.lang = c.lang
               JOIN dynbrain_article a ON a.id = c.article_id
              WHERE c.embedded_at IS NULL AND NOT c.has_error AND a.trashed_at IS NULL
                AND NOT (($2::bool OR ($3::bool AND c.heading_path = ''))
                         AND c.synopsis IS NULL AND NOT c.synopsis_error)
              ORDER BY c.id
              LIMIT $1",
            limit,
            wait_for_all,
            wait_for_headingless
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| EmbedTask {
                id: row.id,
                content: row.content,
                heading_path: row.heading_path,
                synopsis: row.synopsis,
                title: row.title,
                description: row.description,
            })
            .collect())
    }

    pub async fn store_embeddings(
        &self,
        vectors: &[Vectored],
        fingerprint: &str,
    ) -> Result<u64, BrainError> {
        let ids: Vec<i64> = vectors.iter().map(|v| v.id).collect();
        let literals: Vec<String> = vectors.iter().map(|v| literal(&v.vector)).collect();
        let contexts: Vec<String> = vectors.iter().map(|v| v.context_hash.clone()).collect();
        let done = sqlx::query!(
            "UPDATE dynbrain_chunk c
                SET embedding = u.vec::vector, embedding_fingerprint = $4,
                    context_hash = u.context_hash, embedded_at = CURRENT_TIMESTAMP, has_error = false
               FROM UNNEST($1::int8[], $2::text[], $3::text[]) AS u(id, vec, context_hash)
              WHERE c.id = u.id",
            ids.as_slice(),
            literals.as_slice(),
            contexts.as_slice(),
            fingerprint
        )
        .execute(&*self.pool)
        .await?;
        Ok(done.rows_affected())
    }

    pub async fn mark_embedding_errors(&self, ids: &[i64]) -> Result<(), BrainError> {
        sqlx::query!(
            "UPDATE dynbrain_chunk SET has_error = true WHERE id = ANY($1::int8[])",
            ids
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    pub async fn record_embedding_spend(
        &self,
        fingerprint: &str,
        spend: &Spend,
    ) -> Result<(), BrainError> {
        sqlx::query!(
            "INSERT INTO dynbrain_embedding_usage (fingerprint, inputs, input_tokens, cost_microdollars)
             VALUES ($1, $2, $3, $4)",
            fingerprint,
            spend.inputs as i32,
            i64::from(spend.input_tokens),
            spend.cost_microdollars
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    pub async fn fingerprint_counts(&self) -> Result<Vec<FingerprintCount>, BrainError> {
        let rows = sqlx::query_as!(
            FingerprintRow,
            r#"SELECT embedding_fingerprint AS "fingerprint!", COUNT(*) AS "chunks!"
                 FROM dynbrain_chunk
                WHERE embedding_fingerprint IS NOT NULL
                GROUP BY embedding_fingerprint
                ORDER BY COUNT(*) DESC, embedding_fingerprint"#
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| FingerprintCount {
                fingerprint: row.fingerprint,
                chunks: row.chunks,
            })
            .collect())
    }
}

fn literal(vector: &[f32]) -> String {
    let numbers: Vec<String> = vector.iter().map(f32::to_string).collect();
    format!("[{}]", numbers.join(","))
}
