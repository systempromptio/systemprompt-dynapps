//! The indexing queue and what the indexer reads to decide.
//!
//! `dynbrain_index_job` holds one row per variant content (article, language,
//! content hash). A write queues its content; a job that is `done` for the
//! current hash is the record that the chunks match it, and at most one
//! `done` row exists per variant: settling a job removes the others. A claim
//! takes pending jobs of articles that are not in the trash, oldest first,
//! with `SKIP LOCKED`. `enqueue_missing` queues the variants no job covers
//! (variants written before the queue existed, a deleted ledger) and
//! `requeue_index` sends every settled job back, for when the rules change.

use sqlx::PgConnection;

use super::BrainRepository;
use crate::error::BrainError;
use crate::ids::{ArticleId, SpaceId};
use crate::ingest::eligibility::{RagRule, RuleScope};
use crate::ingest::model::{IndexSource, QueuedJob};
use crate::model::RagMode;

struct JobRow {
    id: i64,
    article_id: i64,
    lang: String,
}

struct SourceRow {
    space_id: i64,
    rag_mode: String,
    trashed: bool,
    title: String,
    description: String,
    body: String,
    content_hash: String,
}

struct RuleRow {
    scope: String,
    space_id: Option<i64>,
    article_id: Option<i64>,
    mode: String,
}

pub(crate) async fn enqueue_in(
    conn: &mut PgConnection,
    article: ArticleId,
    lang: &str,
    content_hash: &str,
) -> Result<(), BrainError> {
    sqlx::query!(
        "INSERT INTO dynbrain_index_job (article_id, lang, content_hash) VALUES ($1, $2, $3)
         ON CONFLICT (article_id, lang, content_hash) DO NOTHING",
        article.0,
        lang,
        content_hash
    )
    .execute(conn)
    .await?;
    Ok(())
}

impl BrainRepository {
    pub async fn enqueue_missing(&self, limit: i64) -> Result<u64, BrainError> {
        let done = sqlx::query!(
            "INSERT INTO dynbrain_index_job (article_id, lang, content_hash)
             SELECT v.article_id, v.lang, v.content_hash
               FROM dynbrain_article_variant v
               JOIN dynbrain_article a ON a.id = v.article_id
              WHERE a.trashed_at IS NULL
                AND NOT EXISTS (SELECT 1 FROM dynbrain_index_job j
                                 WHERE j.article_id = v.article_id AND j.lang = v.lang
                                   AND j.content_hash = v.content_hash)
              ORDER BY v.updated_at
              LIMIT $1
             ON CONFLICT (article_id, lang, content_hash) DO NOTHING",
            limit
        )
        .execute(&*self.pool)
        .await?;
        Ok(done.rows_affected())
    }

    pub async fn reset_running(&self) -> Result<u64, BrainError> {
        let done = sqlx::query!(
            "UPDATE dynbrain_index_job SET state = 'pending', updated_at = CURRENT_TIMESTAMP
              WHERE state = 'running'"
        )
        .execute(&*self.pool)
        .await?;
        Ok(done.rows_affected())
    }

    pub async fn requeue_index(&self) -> Result<u64, BrainError> {
        let done = sqlx::query!(
            "UPDATE dynbrain_index_job
                SET state = 'pending', attempts = 0, last_error = NULL, updated_at = CURRENT_TIMESTAMP
              WHERE state IN ('done', 'failed')"
        )
        .execute(&*self.pool)
        .await?;
        Ok(done.rows_affected())
    }

    pub async fn claim_index_jobs(&self, limit: i64) -> Result<Vec<QueuedJob>, BrainError> {
        let rows = sqlx::query_as!(
            JobRow,
            "UPDATE dynbrain_index_job
                SET state = 'running', attempts = attempts + 1, updated_at = CURRENT_TIMESTAMP
              WHERE id IN (SELECT j.id
                             FROM dynbrain_index_job j
                             JOIN dynbrain_article a ON a.id = j.article_id
                            WHERE j.state = 'pending' AND a.trashed_at IS NULL
                            ORDER BY j.created_at, j.id
                            LIMIT $1
                              FOR UPDATE OF j SKIP LOCKED)
              RETURNING id, article_id, lang",
            limit
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| QueuedJob {
                id: row.id,
                article: ArticleId(row.article_id),
                lang: row.lang,
            })
            .collect())
    }

    pub async fn index_source(
        &self,
        article: ArticleId,
        lang: &str,
    ) -> Result<Option<IndexSource>, BrainError> {
        let row = sqlx::query_as!(
            SourceRow,
            r#"SELECT a.space_id, a.rag_mode, (a.trashed_at IS NOT NULL) AS "trashed!",
                      v.title, v.description, v.body, v.content_hash
                 FROM dynbrain_article_variant v
                 JOIN dynbrain_article a ON a.id = v.article_id
                WHERE v.article_id = $1 AND v.lang = $2"#,
            article.0,
            lang
        )
        .fetch_optional(&*self.pool)
        .await?;
        row.map(source).transpose()
    }

    pub async fn lineage(&self, article: ArticleId) -> Result<Vec<ArticleId>, BrainError> {
        let ids = sqlx::query_scalar!(
            r#"WITH RECURSIVE lineage (id, parent_id) AS (
                   SELECT id, parent_id FROM dynbrain_article WHERE id = $1
                   UNION ALL
                   SELECT a.id, a.parent_id FROM dynbrain_article a JOIN lineage l ON a.id = l.parent_id)
               SELECT id AS "id!" FROM lineage"#,
            article.0
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(ids.into_iter().map(ArticleId).collect())
    }

    pub async fn list_rag_rules(&self) -> Result<Vec<RagRule>, BrainError> {
        let rows = sqlx::query_as!(
            RuleRow,
            "SELECT scope, space_id, article_id, mode
               FROM dynbrain_rag_rule WHERE active ORDER BY sequence, id"
        )
        .fetch_all(&*self.pool)
        .await?;
        rows.iter().map(rule).collect()
    }

    pub async fn is_indexed(
        &self,
        article: ArticleId,
        lang: &str,
        content_hash: &str,
    ) -> Result<bool, BrainError> {
        let indexed = sqlx::query_scalar!(
            r#"SELECT EXISTS (SELECT 1 FROM dynbrain_index_job
                               WHERE article_id = $1 AND lang = $2 AND content_hash = $3
                                 AND state = 'done') AS "indexed!""#,
            article.0,
            lang,
            content_hash
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(indexed)
    }

    pub async fn settle_job(&self, job: &QueuedJob, content_hash: &str) -> Result<(), BrainError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            "INSERT INTO dynbrain_index_job (article_id, lang, content_hash, state)
             VALUES ($1, $2, $3, 'done')
             ON CONFLICT (article_id, lang, content_hash)
             DO UPDATE SET state = 'done', last_error = NULL, updated_at = CURRENT_TIMESTAMP",
            job.article.0,
            job.lang,
            content_hash
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query!(
            "DELETE FROM dynbrain_index_job
              WHERE article_id = $1 AND lang = $2 AND content_hash <> $3
                AND (state IN ('done', 'failed') OR id = $4)",
            job.article.0,
            job.lang,
            content_hash,
            job.id
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn fail_job(
        &self,
        job: i64,
        error: &str,
        max_attempts: i32,
    ) -> Result<(), BrainError> {
        sqlx::query!(
            "UPDATE dynbrain_index_job
                SET state = CASE WHEN attempts >= $3 THEN 'failed' ELSE 'pending' END,
                    last_error = $2, updated_at = CURRENT_TIMESTAMP
              WHERE id = $1",
            job,
            error,
            max_attempts
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    pub async fn release_job(&self, job: i64) -> Result<(), BrainError> {
        sqlx::query!(
            "UPDATE dynbrain_index_job SET state = 'pending', updated_at = CURRENT_TIMESTAMP
              WHERE id = $1 AND state = 'running'",
            job
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    pub async fn drop_job(&self, job: i64) -> Result<(), BrainError> {
        sqlx::query!("DELETE FROM dynbrain_index_job WHERE id = $1", job)
            .execute(&*self.pool)
            .await?;
        Ok(())
    }
}

fn source(row: SourceRow) -> Result<IndexSource, BrainError> {
    Ok(IndexSource {
        space: SpaceId(row.space_id),
        rag_mode: RagMode::parse(&row.rag_mode).ok_or(BrainError::Invalid(
            "an article has an unknown retrieval mode",
        ))?,
        trashed: row.trashed,
        title: row.title,
        description: row.description,
        body: row.body,
        content_hash: row.content_hash,
    })
}

fn rule(row: &RuleRow) -> Result<RagRule, BrainError> {
    let scope = match (row.scope.as_str(), row.space_id, row.article_id) {
        ("all", _, _) => RuleScope::All,
        ("space", Some(space), _) => RuleScope::Space(SpaceId(space)),
        ("article", _, Some(article)) => RuleScope::Article(ArticleId(article)),
        _ => return Err(BrainError::Invalid("a retrieval rule names no target")),
    };
    Ok(RagRule {
        scope,
        include: row.mode == "include",
    })
}
