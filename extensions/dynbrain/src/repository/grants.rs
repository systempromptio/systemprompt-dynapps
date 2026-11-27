//! Grants, inheritance switches and the access they resolve to.
//!
//! Writing or revoking a grant rewrites the access closure of its target (the
//! whole space for a space grant, the article and its subtree otherwise) in
//! the same transaction, and so does turning an article's `inherit_access`
//! on or off. Turning it off with `copy_inherited` first writes what the
//! article inherited as its own grants (an own grant for the same subject is
//! kept), so the article keeps the same access and stops following its
//! parent; without it the article just cuts.
//!
//! [`BrainRepository::find_space_level`] is the same question for a space
//! itself: its owner and the `admin` role manage it, and every other user
//! reaches it through the space-level grants for them, their roles or their
//! groups.
//!
//! [`BrainRepository::find_level`] reads only the closure, as retrieval will:
//! a user reaches an article through a row for them, for one of their roles
//! or for one of their groups, and only while their account is active.

use systemprompt::identifiers::UserId;

use super::types::{AccessRow, ArticleRow, GrantRow};
use super::{BrainRepository, list_space_grants, list_tree, lock_space, rewrite_access};
use crate::access::{self, AccessEntry, Grant, Level, Subject};
use crate::error::BrainError;
use crate::ids::{ArticleId, SpaceId};
use crate::model::Article;

impl BrainRepository {
    pub async fn set_grant(&self, grant: &Grant, granted_by: &UserId) -> Result<(), BrainError> {
        if !grant.subject.is_valid() {
            return Err(BrainError::Invalid(
                "a grant subject must be 1 to 255 characters",
            ));
        }
        let mut tx = self.pool.begin().await?;
        if let Some(article) = grant.article
            && space_of(&mut tx, article).await? != grant.space
        {
            return Err(BrainError::Invalid(
                "the article of a grant must be in the grant's space",
            ));
        }
        let owner = lock_space(&mut tx, grant.space).await?;
        sqlx::query!(
            "INSERT INTO dynbrain_grant (space_id, article_id, subject_kind, subject_id, level, granted_by)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (space_id, article_id, subject_kind, subject_id) DO UPDATE SET
                    level = EXCLUDED.level,
                    granted_by = EXCLUDED.granted_by,
                    updated_at = CURRENT_TIMESTAMP",
            grant.space.0,
            grant.article.map(|article| article.0),
            grant.subject.kind.as_db(),
            grant.subject.id,
            grant.level.as_db(),
            granted_by.as_str()
        )
        .execute(&mut *tx)
        .await?;
        rewrite_access(&mut tx, grant.space, owner.as_ref(), grant.article).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn revoke_grant(
        &self,
        space: SpaceId,
        article: Option<ArticleId>,
        subject: &Subject,
    ) -> Result<bool, BrainError> {
        let mut tx = self.pool.begin().await?;
        let owner = lock_space(&mut tx, space).await?;
        let removed = sqlx::query!(
            "DELETE FROM dynbrain_grant
              WHERE space_id = $1 AND article_id IS NOT DISTINCT FROM $2
                AND subject_kind = $3 AND subject_id = $4",
            space.0,
            article.map(|article| article.0),
            subject.kind.as_db(),
            subject.id
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();
        rewrite_access(&mut tx, space, owner.as_ref(), article).await?;
        tx.commit().await?;
        Ok(removed > 0)
    }

    pub async fn list_grants(
        &self,
        space: SpaceId,
        article: Option<ArticleId>,
    ) -> Result<Vec<Grant>, BrainError> {
        sqlx::query_as!(
            GrantRow,
            "SELECT space_id, article_id, subject_kind, subject_id, level
               FROM dynbrain_grant
              WHERE space_id = $1 AND article_id IS NOT DISTINCT FROM $2
              ORDER BY subject_kind, subject_id",
            space.0,
            article.map(|article| article.0)
        )
        .fetch_all(&*self.pool)
        .await?
        .into_iter()
        .map(Grant::try_from)
        .collect()
    }

    pub async fn set_inherit(
        &self,
        article: ArticleId,
        inherit: bool,
        copy_inherited: bool,
        changed_by: &UserId,
    ) -> Result<Article, BrainError> {
        let mut tx = self.pool.begin().await?;
        let space = space_of(&mut tx, article).await?;
        let owner = lock_space(&mut tx, space).await?;
        if !inherit && copy_inherited {
            let nodes = list_tree(&mut tx, space).await?;
            let grants = list_space_grants(&mut tx, space).await?;
            let copied = access::inherited(&nodes, &grants, article);
            let kinds: Vec<String> = copied
                .iter()
                .map(|(subject, _)| subject.kind.as_db().to_owned())
                .collect();
            let subjects: Vec<String> = copied
                .iter()
                .map(|(subject, _)| subject.id.clone())
                .collect();
            let levels: Vec<i16> = copied.iter().map(|(_, level)| level.as_db()).collect();
            sqlx::query!(
                "INSERT INTO dynbrain_grant (space_id, article_id, subject_kind, subject_id, level, granted_by)
                 SELECT $1::int8, $2::int8, copied.kind, copied.subject, copied.level, $6::text
                   FROM UNNEST($3::text[], $4::text[], $5::int2[]) AS copied(kind, subject, level)
                 ON CONFLICT (space_id, article_id, subject_kind, subject_id) DO NOTHING",
                space.0,
                article.0,
                kinds.as_slice(),
                subjects.as_slice(),
                levels.as_slice(),
                changed_by.as_str()
            )
            .execute(&mut *tx)
            .await?;
        }
        let row = sqlx::query_as!(
            ArticleRow,
            "UPDATE dynbrain_article SET inherit_access = $2, updated_at = CURRENT_TIMESTAMP
              WHERE id = $1
             RETURNING id, space_id, parent_id, position, slug, icon, rag_mode, inherit_access, created_by,
                       created_at, updated_at, trashed_at",
            article.0,
            inherit
        )
        .fetch_one(&mut *tx)
        .await?;
        let updated = Article::try_from(row)?;
        rewrite_access(&mut tx, space, owner.as_ref(), Some(article)).await?;
        tx.commit().await?;
        Ok(updated)
    }

    pub async fn list_access(&self, article: ArticleId) -> Result<Vec<AccessEntry>, BrainError> {
        sqlx::query_as!(
            AccessRow,
            "SELECT article_id, subject_kind, subject_id, level
               FROM dynbrain_access WHERE article_id = $1
              ORDER BY subject_kind, subject_id",
            article.0
        )
        .fetch_all(&*self.pool)
        .await?
        .into_iter()
        .map(AccessEntry::try_from)
        .collect()
    }

    pub async fn find_level(
        &self,
        article: ArticleId,
        user: &UserId,
    ) -> Result<Option<Level>, BrainError> {
        let level = sqlx::query_scalar!(
            r#"SELECT MAX(x.level) AS "level"
                 FROM dynbrain_access x
                 JOIN users u ON u.id = $2 AND u.status = 'active'
                WHERE x.article_id = $1
                  AND ((x.subject_kind = 'user' AND x.subject_id = u.id)
                       OR (x.subject_kind = 'role' AND x.subject_id = ANY(u.roles))
                       OR (x.subject_kind = 'group' AND x.subject_id IN
                               (SELECT g.group_id FROM user_groups g WHERE g.user_id = u.id)))"#,
            article.0,
            user.as_str()
        )
        .fetch_one(&*self.pool)
        .await?;
        level
            .map(|stored| {
                Level::from_db(stored).ok_or(BrainError::Invalid("a stored level is unknown"))
            })
            .transpose()
    }

    pub async fn find_space_level(
        &self,
        space: SpaceId,
        user: &UserId,
    ) -> Result<Option<Level>, BrainError> {
        let level = sqlx::query_scalar!(
            r#"SELECT MAX(CASE WHEN s.created_by = u.id OR 'admin' = ANY(u.roles) THEN 3::int2 ELSE g.level END) AS "level"
                 FROM dynbrain_space s
                 JOIN users u ON u.id = $2 AND u.status = 'active'
                 LEFT JOIN dynbrain_grant g ON g.space_id = s.id AND g.article_id IS NULL
                  AND ((g.subject_kind = 'user' AND g.subject_id = u.id)
                       OR (g.subject_kind = 'role' AND g.subject_id = ANY(u.roles))
                       OR (g.subject_kind = 'group' AND g.subject_id IN
                               (SELECT m.group_id FROM user_groups m WHERE m.user_id = u.id)))
                WHERE s.id = $1"#,
            space.0,
            user.as_str()
        )
        .fetch_one(&*self.pool)
        .await?;
        level
            .map(|stored| {
                Level::from_db(stored).ok_or(BrainError::Invalid("a stored level is unknown"))
            })
            .transpose()
    }
}

async fn space_of(
    conn: &mut sqlx::PgConnection,
    article: ArticleId,
) -> Result<SpaceId, BrainError> {
    let space = sqlx::query_scalar!(
        "SELECT space_id FROM dynbrain_article WHERE id = $1",
        article.0
    )
    .fetch_optional(&mut *conn)
    .await?
    .ok_or(BrainError::ArticleNotFound(article))?;
    Ok(SpaceId(space))
}
