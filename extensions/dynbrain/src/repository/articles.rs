//! Articles: create, read, list, move, trash and restore.
//!
//! Creating and moving rewrite the access closure of the articles they
//! affect (the new article; the moved article and its subtree) in the same
//! transaction.

use chrono::Utc;
use systemprompt::identifiers::UserId;

use super::types::ArticleRow;
use super::{BrainRepository, list_tree, lock_space, rewrite_access};
use crate::access::subtree;
use crate::error::BrainError;
use crate::ids::{ArticleId, SpaceId};
use crate::model::{Article, ArticleMove, NewArticle};

impl BrainRepository {
    pub async fn insert_article(
        &self,
        article: &NewArticle,
        author: &UserId,
    ) -> Result<Article, BrainError> {
        let mut tx = self.pool.begin().await?;
        let owner = lock_space(&mut tx, article.space).await?;
        if let Some(parent) = article.parent {
            let found = find_article_in(&mut tx, parent).await?;
            let parent_row = found.ok_or(BrainError::ArticleNotFound(parent))?;
            if parent_row.space != article.space {
                return Err(BrainError::MoveAcrossSpaces(parent));
            }
            if parent_row.trashed_at.is_some() {
                return Err(BrainError::Trashed(parent));
            }
        }
        let row = sqlx::query_as!(
            ArticleRow,
            "INSERT INTO dynbrain_article (space_id, parent_id, position, slug, icon, rag_mode, created_by)
             VALUES ($1, $2, COALESCE($3::int4, (SELECT COALESCE(MAX(position) + 1, 0)
                                                   FROM dynbrain_article
                                                  WHERE space_id = $1
                                                    AND parent_id IS NOT DISTINCT FROM $2)),
                     $4, $5, $6, $7)
             RETURNING id, space_id, parent_id, position, slug, icon, rag_mode, inherit_access, created_by, created_at,
                       updated_at, trashed_at",
            article.space.0,
            article.parent.map(|parent| parent.0),
            article.position,
            article.slug,
            article.icon,
            article.rag_mode.as_db(),
            author.as_str()
        )
        .fetch_one(&mut *tx)
        .await?;
        let created = Article::try_from(row)?;
        rewrite_access(&mut tx, article.space, owner.as_ref(), Some(created.id)).await?;
        tx.commit().await?;
        Ok(created)
    }

    pub async fn find_article(&self, article: ArticleId) -> Result<Option<Article>, BrainError> {
        let mut conn = self.pool.acquire().await?;
        find_article_in(&mut conn, article).await
    }

    pub async fn list_articles(&self, space: SpaceId) -> Result<Vec<Article>, BrainError> {
        sqlx::query_as!(
            ArticleRow,
            "SELECT id, space_id, parent_id, position, slug, icon, rag_mode, inherit_access, created_by, created_at,
                    updated_at, trashed_at
               FROM dynbrain_article WHERE space_id = $1
              ORDER BY parent_id NULLS FIRST, position, id",
            space.0
        )
        .fetch_all(&*self.pool)
        .await?
        .into_iter()
        .map(Article::try_from)
        .collect()
    }

    pub async fn list_children(
        &self,
        space: SpaceId,
        parent: Option<ArticleId>,
    ) -> Result<Vec<Article>, BrainError> {
        sqlx::query_as!(
            ArticleRow,
            "SELECT id, space_id, parent_id, position, slug, icon, rag_mode, inherit_access, created_by, created_at,
                    updated_at, trashed_at
               FROM dynbrain_article
              WHERE space_id = $1 AND parent_id IS NOT DISTINCT FROM $2 AND trashed_at IS NULL
              ORDER BY position, id",
            space.0,
            parent.map(|parent| parent.0)
        )
        .fetch_all(&*self.pool)
        .await?
        .into_iter()
        .map(Article::try_from)
        .collect()
    }

    pub async fn move_article(
        &self,
        article: ArticleId,
        target: ArticleMove,
    ) -> Result<Article, BrainError> {
        let mut tx = self.pool.begin().await?;
        let current = find_article_in(&mut tx, article)
            .await?
            .ok_or(BrainError::ArticleNotFound(article))?;
        let owner = lock_space(&mut tx, current.space).await?;
        if let Some(parent) = target.parent {
            let nodes = list_tree(&mut tx, current.space).await?;
            if subtree(&nodes, article).contains(&parent) {
                return Err(BrainError::MoveIntoOwnSubtree(article));
            }
            if !nodes.iter().any(|node| node.id == parent) {
                let exists = find_article_in(&mut tx, parent).await?.is_some();
                return Err(if exists {
                    BrainError::MoveAcrossSpaces(article)
                } else {
                    BrainError::ArticleNotFound(parent)
                });
            }
        }
        let row = sqlx::query_as!(
            ArticleRow,
            "UPDATE dynbrain_article
                SET parent_id = $2,
                    position = COALESCE($3::int4, (SELECT COALESCE(MAX(position) + 1, 0)
                                                     FROM dynbrain_article
                                                    WHERE space_id = $4
                                                      AND parent_id IS NOT DISTINCT FROM $2
                                                      AND id <> $1)),
                    updated_at = CURRENT_TIMESTAMP
              WHERE id = $1
             RETURNING id, space_id, parent_id, position, slug, icon, rag_mode, inherit_access, created_by, created_at,
                       updated_at, trashed_at",
            article.0,
            target.parent.map(|parent| parent.0),
            target.position,
            current.space.0
        )
        .fetch_one(&mut *tx)
        .await?;
        let moved = Article::try_from(row)?;
        rewrite_access(&mut tx, current.space, owner.as_ref(), Some(article)).await?;
        tx.commit().await?;
        Ok(moved)
    }

    pub async fn trash_article(&self, article: ArticleId) -> Result<Article, BrainError> {
        self.set_trashed(article, true).await
    }

    pub async fn restore_article(&self, article: ArticleId) -> Result<Article, BrainError> {
        self.set_trashed(article, false).await
    }

    async fn set_trashed(&self, article: ArticleId, trash: bool) -> Result<Article, BrainError> {
        let mut tx = self.pool.begin().await?;
        let current = find_article_in(&mut tx, article)
            .await?
            .ok_or(BrainError::ArticleNotFound(article))?;
        lock_space(&mut tx, current.space).await?;
        let nodes = list_tree(&mut tx, current.space).await?;
        let ids: Vec<i64> = subtree(&nodes, article).iter().map(|id| id.0).collect();
        if trash {
            sqlx::query!(
                "UPDATE dynbrain_article SET trashed_at = $2, updated_at = $2
                  WHERE id = ANY($1::int8[]) AND trashed_at IS NULL",
                ids.as_slice(),
                Utc::now()
            )
            .execute(&mut *tx)
            .await?;
        } else if let Some(trashed_at) = current.trashed_at {
            sqlx::query!(
                "UPDATE dynbrain_article SET trashed_at = NULL, updated_at = CURRENT_TIMESTAMP
                  WHERE id = ANY($1::int8[]) AND trashed_at = $2",
                ids.as_slice(),
                trashed_at
            )
            .execute(&mut *tx)
            .await?;
        }
        let updated = find_article_in(&mut tx, article)
            .await?
            .ok_or(BrainError::ArticleNotFound(article))?;
        tx.commit().await?;
        Ok(updated)
    }
}

async fn find_article_in(
    conn: &mut sqlx::PgConnection,
    article: ArticleId,
) -> Result<Option<Article>, BrainError> {
    sqlx::query_as!(
        ArticleRow,
        "SELECT id, space_id, parent_id, position, slug, icon, rag_mode, inherit_access, created_by, created_at,
                updated_at, trashed_at
           FROM dynbrain_article WHERE id = $1",
        article.0
    )
    .fetch_optional(&mut *conn)
    .await?
    .map(Article::try_from)
    .transpose()
}
