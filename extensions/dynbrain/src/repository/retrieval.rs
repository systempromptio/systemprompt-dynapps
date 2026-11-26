//! The reads of retrieval, with the reader's access inside each query.
//!
//! A chunk or a document counts only when the person reaches its article
//! through the access closure (a row for them, one of their roles or one of
//! their groups, at `read` or more, while their account is active) and the
//! article is out of the trash. The filter is part of the SQL, never applied
//! afterwards, so a reader who sees little still gets a full pool of what they
//! may read, and nothing they may not read is ever ranked.
//!
//! Both arms also honour the scope: one space, the spaces written in a
//! language, and the `where` filter (carried values compared as JSONB, unset
//! keys absent, `false`, `null` or empty).

use systemprompt::identifiers::UserId;

use super::BrainRepository;
use crate::access::Level;
use crate::error::BrainError;
use crate::ids::{ArticleId, ChunkId, SpaceId};
use crate::search::query::QueryVector;
use crate::search::service::{Passage, SearchScope};

/// An article a person reads, with its space and the level they reach.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReachedArticle {
    pub article: ArticleId,
    pub space: SpaceId,
    pub level: Level,
}

struct ReachRow {
    id: i64,
    space_id: i64,
    level: i16,
}

struct KeywordRow {
    article_id: i64,
    space_id: i64,
    lang: String,
    title: String,
    description: String,
    snippet: String,
}

impl BrainRepository {
    pub async fn list_article_levels(
        &self,
        user: &UserId,
        space: Option<SpaceId>,
    ) -> Result<Vec<ReachedArticle>, BrainError> {
        let rows = sqlx::query_as!(
            ReachRow,
            r#"SELECT a.id AS "id!", a.space_id AS "space_id!", MAX(x.level) AS "level!"
                 FROM dynbrain_access x
                 JOIN dynbrain_article a ON a.id = x.article_id AND a.trashed_at IS NULL
                 JOIN users u ON u.id = $1 AND u.status = 'active'
                WHERE ($2::int8 IS NULL OR a.space_id = $2)
                  AND x.level >= 1
                  AND ((x.subject_kind = 'user' AND x.subject_id = u.id)
                       OR (x.subject_kind = 'role' AND x.subject_id = ANY(u.roles))
                       OR (x.subject_kind = 'group' AND x.subject_id IN
                               (SELECT g.group_id FROM user_groups g WHERE g.user_id = u.id)))
                GROUP BY a.id, a.space_id"#,
            user.as_str(),
            space.map(|space| space.0)
        )
        .fetch_all(&*self.pool)
        .await?;
        rows.into_iter()
            .map(|row| {
                Ok(ReachedArticle {
                    article: ArticleId(row.id),
                    space: SpaceId(row.space_id),
                    level: Level::from_db(row.level)
                        .ok_or(BrainError::Invalid("a stored level is unknown"))?,
                })
            })
            .collect()
    }

    pub async fn list_vector_matches(
        &self,
        scope: &SearchScope,
        vector: &QueryVector,
        pool: i64,
        effort: i64,
    ) -> Result<Vec<ChunkId>, BrainError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            "SELECT set_config('hnsw.ef_search', $1, true)",
            effort.to_string()
        )
        .execute(&mut *tx)
        .await?;
        let ids = sqlx::query_scalar!(
            r#"WITH readable AS (
                   SELECT x.article_id
                     FROM dynbrain_access x
                     JOIN users u ON u.id = $1 AND u.status = 'active'
                    WHERE x.level >= 1
                      AND ((x.subject_kind = 'user' AND x.subject_id = u.id)
                           OR (x.subject_kind = 'role' AND x.subject_id = ANY(u.roles))
                           OR (x.subject_kind = 'group' AND x.subject_id IN
                                   (SELECT g.group_id FROM user_groups g WHERE g.user_id = u.id)))
               )
               SELECT c.id AS "id!"
                 FROM dynbrain_chunk c
                 JOIN dynbrain_article a ON a.id = c.article_id AND a.trashed_at IS NULL
                 JOIN dynbrain_space s ON s.id = a.space_id
                 JOIN dynbrain_article_variant v ON v.article_id = c.article_id AND v.lang = c.lang
                WHERE c.embedding IS NOT NULL
                  AND c.article_id IN (SELECT r.article_id FROM readable r)
                  AND ($2::int8 IS NULL OR a.space_id = $2)
                  AND ($3::text IS NULL OR $3 = ANY(s.langs))
                  AND NOT EXISTS (SELECT 1 FROM jsonb_each($4::jsonb) AS w(key, value)
                                   WHERE v.properties -> w.key IS DISTINCT FROM w.value)
                  AND NOT EXISTS (SELECT 1 FROM UNNEST($5::text[]) AS k(name)
                                   WHERE COALESCE(v.properties ->> k.name, 'false') NOT IN ('false', ''))
                ORDER BY c.embedding <=> $6::text::vector, c.id
                LIMIT $7"#,
            scope.reader.as_str(),
            scope.space.map(|space| space.0),
            scope.lang.as_deref(),
            scope.filter.carried_document(),
            scope.filter.unset.as_slice(),
            vector.0.as_str(),
            pool
        )
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(ids.into_iter().map(ChunkId).collect())
    }

    pub async fn list_text_matches(
        &self,
        scope: &SearchScope,
        query: &str,
        pool: i64,
    ) -> Result<Vec<ChunkId>, BrainError> {
        let ids = sqlx::query_scalar!(
            r#"WITH readable AS (
                   SELECT x.article_id
                     FROM dynbrain_access x
                     JOIN users u ON u.id = $1 AND u.status = 'active'
                    WHERE x.level >= 1
                      AND ((x.subject_kind = 'user' AND x.subject_id = u.id)
                           OR (x.subject_kind = 'role' AND x.subject_id = ANY(u.roles))
                           OR (x.subject_kind = 'group' AND x.subject_id IN
                                   (SELECT g.group_id FROM user_groups g WHERE g.user_id = u.id)))
               )
               SELECT c.id AS "id!"
                 FROM dynbrain_chunk c
                 JOIN dynbrain_article a ON a.id = c.article_id AND a.trashed_at IS NULL
                 JOIN dynbrain_space s ON s.id = a.space_id
                 JOIN dynbrain_article_variant v ON v.article_id = c.article_id AND v.lang = c.lang
                WHERE c.search_vector @@ plainto_tsquery(c.fts_config, $6::text)
                  AND c.article_id IN (SELECT r.article_id FROM readable r)
                  AND ($2::int8 IS NULL OR a.space_id = $2)
                  AND ($3::text IS NULL OR $3 = ANY(s.langs))
                  AND NOT EXISTS (SELECT 1 FROM jsonb_each($4::jsonb) AS w(key, value)
                                   WHERE v.properties -> w.key IS DISTINCT FROM w.value)
                  AND NOT EXISTS (SELECT 1 FROM UNNEST($5::text[]) AS k(name)
                                   WHERE COALESCE(v.properties ->> k.name, 'false') NOT IN ('false', ''))
                ORDER BY ts_rank(c.search_vector, plainto_tsquery(c.fts_config, $6::text)) DESC, c.id
                LIMIT $7"#,
            scope.reader.as_str(),
            scope.space.map(|space| space.0),
            scope.lang.as_deref(),
            scope.filter.carried_document(),
            scope.filter.unset.as_slice(),
            query,
            pool
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(ids.into_iter().map(ChunkId).collect())
    }

    pub async fn list_keyword_matches(
        &self,
        scope: &SearchScope,
        pattern: &str,
        limit: i64,
    ) -> Result<Vec<Passage>, BrainError> {
        let rows = sqlx::query_as!(
            KeywordRow,
            r#"WITH readable AS (
                   SELECT x.article_id
                     FROM dynbrain_access x
                     JOIN users u ON u.id = $1 AND u.status = 'active'
                    WHERE x.level >= 1
                      AND ((x.subject_kind = 'user' AND x.subject_id = u.id)
                           OR (x.subject_kind = 'role' AND x.subject_id = ANY(u.roles))
                           OR (x.subject_kind = 'group' AND x.subject_id IN
                                   (SELECT g.group_id FROM user_groups g WHERE g.user_id = u.id)))
               )
               SELECT DISTINCT ON (a.id)
                      a.id AS "article_id!", a.space_id AS "space_id!", v.lang AS "lang!",
                      v.title AS "title!", v.description AS "description!",
                      LEFT(v.body, 300) AS "snippet!"
                 FROM dynbrain_article_variant v
                 JOIN dynbrain_article a ON a.id = v.article_id AND a.trashed_at IS NULL
                 JOIN dynbrain_space s ON s.id = a.space_id
                WHERE a.id IN (SELECT r.article_id FROM readable r)
                  AND ($2::int8 IS NULL OR a.space_id = $2)
                  AND ($3::text IS NULL OR v.lang = $3)
                  AND NOT EXISTS (SELECT 1 FROM jsonb_each($4::jsonb) AS w(key, value)
                                   WHERE v.properties -> w.key IS DISTINCT FROM w.value)
                  AND NOT EXISTS (SELECT 1 FROM UNNEST($5::text[]) AS k(name)
                                   WHERE COALESCE(v.properties ->> k.name, 'false') NOT IN ('false', ''))
                  AND (v.title ILIKE $6::text OR v.body ILIKE $6::text)
                ORDER BY a.id, v.lang = s.default_lang DESC, v.lang
                LIMIT $7"#,
            scope.reader.as_str(),
            scope.space.map(|space| space.0),
            scope.lang.as_deref(),
            scope.filter.carried_document(),
            scope.filter.unset.as_slice(),
            pattern,
            limit
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| Passage {
                chunk: None,
                article: ArticleId(row.article_id),
                space: SpaceId(row.space_id),
                lang: row.lang,
                title: row.title,
                description: row.description,
                content: row.snippet,
                score: None,
            })
            .collect())
    }
}
