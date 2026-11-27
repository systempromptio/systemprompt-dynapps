//! The Brain's usage: one row per search or answer (counts only), the
//! admin's audit and what they add up to.
//!
//! [`BrainRepository::record_search`] is the one write here the operations
//! call, after they answer.

use chrono::{DateTime, Utc};

use super::BrainRepository;
use crate::error::BrainError;
use crate::ids::ArticleId;

/// How often an article was returned by a search and cited by an answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsedArticle {
    pub article_id: i64,
    pub title: String,
    pub retrieved: i64,
    pub cited: i64,
}

/// A query that found nothing, with how many times it was asked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmptySearch {
    pub query: String,
    pub space: Option<String>,
    pub times: i64,
}

/// What the embeddings of one configuration cost.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddingSpend {
    pub fingerprint: String,
    pub calls: i64,
    pub micros: i64,
}

/// The uses of one article in a period.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArticleUsage {
    pub retrieved: i64,
    pub cited: i64,
}

impl BrainRepository {
    pub async fn record_search(
        &self,
        kind: &str,
        query: &str,
        space: Option<i64>,
        articles: &[i64],
    ) -> Result<(), BrainError> {
        sqlx::query!(
            "INSERT INTO dynbrain_search_log (kind, query_text, space_id, article_ids)
             VALUES ($1, $2, $3, $4)",
            kind,
            query,
            space,
            articles
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    pub async fn record_admin_action(
        &self,
        actor: &str,
        action: &str,
        subject: Option<&str>,
        detail: &str,
    ) -> Result<(), BrainError> {
        sqlx::query!(
            "INSERT INTO dynbrain_admin_audit (actor_id, action, subject_id, detail)
             VALUES ($1, $2, $3, $4)",
            actor,
            action,
            subject,
            detail
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    pub async fn count_searches(&self, since: DateTime<Utc>) -> Result<i64, BrainError> {
        Ok(sqlx::query_scalar!(
            r#"SELECT count(*) AS "n!" FROM dynbrain_search_log WHERE created_at >= $1"#,
            since
        )
        .fetch_one(&*self.pool)
        .await?)
    }

    pub async fn list_used_articles(
        &self,
        since: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<UsedArticle>, BrainError> {
        Ok(sqlx::query_as!(
            UsedArticle,
            r#"SELECT t.article_id AS "article_id!",
                      COALESCE((SELECT v.title FROM dynbrain_article_variant v
                                 WHERE v.article_id = t.article_id ORDER BY v.lang LIMIT 1), '') AS "title!",
                      count(*) FILTER (WHERE t.kind = 'search') AS "retrieved!",
                      count(*) FILTER (WHERE t.kind = 'answer') AS "cited!"
                 FROM (SELECT l.kind, unnest(l.article_ids) AS article_id
                         FROM dynbrain_search_log l WHERE l.created_at >= $1) t
                GROUP BY t.article_id
                ORDER BY count(*) DESC, t.article_id
                LIMIT $2"#,
            since,
            limit
        )
        .fetch_all(&*self.pool)
        .await?)
    }

    pub async fn list_empty_searches(
        &self,
        since: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<EmptySearch>, BrainError> {
        Ok(sqlx::query_as!(
            EmptySearch,
            r#"SELECT l.query_text AS "query!", s.slug AS "space?", count(*) AS "times!"
                 FROM dynbrain_search_log l
                 LEFT JOIN dynbrain_space s ON s.id = l.space_id
                WHERE l.created_at >= $1 AND cardinality(l.article_ids) = 0
                GROUP BY l.query_text, s.slug
                ORDER BY count(*) DESC, l.query_text
                LIMIT $2"#,
            since,
            limit
        )
        .fetch_all(&*self.pool)
        .await?)
    }

    pub async fn list_embedding_spend(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<EmbeddingSpend>, BrainError> {
        Ok(sqlx::query_as!(
            EmbeddingSpend,
            r#"SELECT fingerprint, count(*) AS "calls!",
                      COALESCE(sum(cost_microdollars), 0)::bigint AS "micros!"
                 FROM dynbrain_embedding_usage
                WHERE created_at >= $1
                GROUP BY fingerprint
                ORDER BY 3 DESC, fingerprint"#,
            since
        )
        .fetch_all(&*self.pool)
        .await?)
    }

    pub async fn count_article_usage(
        &self,
        article: ArticleId,
        since: DateTime<Utc>,
    ) -> Result<ArticleUsage, BrainError> {
        Ok(sqlx::query_as!(
            ArticleUsage,
            r#"SELECT count(*) FILTER (WHERE kind = 'search') AS "retrieved!",
                      count(*) FILTER (WHERE kind = 'answer') AS "cited!"
                 FROM dynbrain_search_log
                WHERE created_at >= $1 AND $2 = ANY(article_ids)"#,
            since,
            article.0
        )
        .fetch_one(&*self.pool)
        .await?)
    }
}
