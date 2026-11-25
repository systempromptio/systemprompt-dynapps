//! Grants and the access they resolve to.
//!
//! Writing or revoking a grant rewrites the access closure of the article
//! and its subtree in the same transaction. [`BrainRepository::find_level`]
//! reads only the closure, as retrieval will: a user reaches an article
//! through a row for them, for one of their roles or for one of their groups,
//! and only while their account is active.

use systemprompt::identifiers::UserId;

use super::types::{AccessRow, GrantRow};
use super::{BrainRepository, lock_space, rewrite_access};
use crate::access::{AccessEntry, Grant, Level, Subject};
use crate::error::BrainError;
use crate::ids::{ArticleId, SpaceId};

impl BrainRepository {
    pub async fn set_grant(&self, grant: &Grant, granted_by: &UserId) -> Result<(), BrainError> {
        if !grant.subject.is_valid() {
            return Err(BrainError::Invalid(
                "a grant subject must be 1 to 255 characters",
            ));
        }
        let mut tx = self.pool.begin().await?;
        let space = space_of(&mut tx, grant.article).await?;
        let owner = lock_space(&mut tx, space).await?;
        sqlx::query!(
            "INSERT INTO dynbrain_grant (article_id, subject_kind, subject_id, level, inherit, granted_by)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (article_id, subject_kind, subject_id) DO UPDATE SET
                    level = EXCLUDED.level,
                    inherit = EXCLUDED.inherit,
                    granted_by = EXCLUDED.granted_by,
                    updated_at = CURRENT_TIMESTAMP",
            grant.article.0,
            grant.subject.kind.as_db(),
            grant.subject.id,
            grant.level.as_db(),
            grant.inherit,
            granted_by.as_str()
        )
        .execute(&mut *tx)
        .await?;
        rewrite_access(&mut tx, space, owner.as_ref(), grant.article).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn revoke_grant(
        &self,
        article: ArticleId,
        subject: &Subject,
    ) -> Result<bool, BrainError> {
        let mut tx = self.pool.begin().await?;
        let space = space_of(&mut tx, article).await?;
        let owner = lock_space(&mut tx, space).await?;
        let removed = sqlx::query!(
            "DELETE FROM dynbrain_grant WHERE article_id = $1 AND subject_kind = $2 AND subject_id = $3",
            article.0,
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

    pub async fn list_grants(&self, article: ArticleId) -> Result<Vec<Grant>, BrainError> {
        sqlx::query_as!(
            GrantRow,
            "SELECT article_id, subject_kind, subject_id, level, inherit
               FROM dynbrain_grant WHERE article_id = $1
              ORDER BY subject_kind, subject_id",
            article.0
        )
        .fetch_all(&*self.pool)
        .await?
        .into_iter()
        .map(Grant::try_from)
        .collect()
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
