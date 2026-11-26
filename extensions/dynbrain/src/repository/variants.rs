//! Variants and their revisions.
//!
//! A write replaces the variant's whole content, checks the property values
//! against the space's schema, bumps the variant's revision and appends the
//! written content to `dynbrain_article_revision` and queues the content for
//! indexing (`dynbrain_index_job`), all in one transaction.

// JSON: the front matter and the property values are written as JSON objects.
use serde_json::Value;
use systemprompt::identifiers::UserId;

use super::BrainRepository;
use super::ingest::enqueue_in;
use super::types::{RevisionRow, VariantRow, VariantTargetRow};
use crate::content::content_hash;
use crate::error::BrainError;
use crate::ids::{ArticleId, SpaceId};
use crate::model::{Revision, Variant, VariantWrite};
use crate::properties::PropertySchema;

impl BrainRepository {
    pub async fn write_variant(
        &self,
        article: ArticleId,
        lang: &str,
        content: &VariantWrite,
        author: &UserId,
    ) -> Result<Variant, BrainError> {
        if content.title.trim().is_empty() {
            return Err(BrainError::Invalid("a variant needs a title"));
        }
        let mut tx = self.pool.begin().await?;
        let target = sqlx::query_as!(
            VariantTargetRow,
            "SELECT a.space_id, s.langs, s.property_schema
               FROM dynbrain_article a
               JOIN dynbrain_space s ON s.id = a.space_id
              WHERE a.id = $1 AND a.trashed_at IS NULL
                FOR UPDATE OF a",
            article.0
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(BrainError::ArticleNotFound(article))?;
        if !target.langs.iter().any(|known| known == lang) {
            return Err(BrainError::LanguageNotInSpace {
                lang: lang.to_owned(),
                space: SpaceId(target.space_id),
            });
        }
        PropertySchema::parse(&target.property_schema)?.validate(&content.properties)?;
        let hash = content_hash(&content.title, &content.description, &content.body);
        let row = sqlx::query_as!(
            VariantRow,
            "INSERT INTO dynbrain_article_variant
                    (article_id, lang, title, description, body, front_matter, properties, content_hash,
                     updated_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             ON CONFLICT (article_id, lang) DO UPDATE SET
                    title = EXCLUDED.title,
                    description = EXCLUDED.description,
                    body = EXCLUDED.body,
                    front_matter = EXCLUDED.front_matter,
                    properties = EXCLUDED.properties,
                    content_hash = EXCLUDED.content_hash,
                    updated_by = EXCLUDED.updated_by,
                    revision = dynbrain_article_variant.revision + 1,
                    updated_at = CURRENT_TIMESTAMP
             RETURNING article_id, lang, title, description, body, front_matter, properties,
                       content_hash, revision, updated_by, created_at, updated_at",
            article.0,
            lang,
            content.title,
            content.description,
            content.body,
            Value::Object(content.front_matter.clone()),
            Value::Object(content.properties.clone()),
            hash,
            author.as_str()
        )
        .fetch_one(&mut *tx)
        .await?;
        sqlx::query!(
            "INSERT INTO dynbrain_article_revision
                    (article_id, lang, revision, title, description, body, front_matter, properties,
                     content_hash, author_id)
             SELECT article_id, lang, revision, title, description, body, front_matter, properties,
                    content_hash, $3
               FROM dynbrain_article_variant
              WHERE article_id = $1 AND lang = $2",
            article.0,
            lang,
            author.as_str()
        )
        .execute(&mut *tx)
        .await?;
        enqueue_in(&mut tx, article, lang, &hash).await?;
        let variant = Variant::try_from(row)?;
        tx.commit().await?;
        Ok(variant)
    }

    pub async fn find_variant(
        &self,
        article: ArticleId,
        lang: &str,
    ) -> Result<Option<Variant>, BrainError> {
        sqlx::query_as!(
            VariantRow,
            "SELECT article_id, lang, title, description, body, front_matter, properties,
                    content_hash, revision, updated_by, created_at, updated_at
               FROM dynbrain_article_variant WHERE article_id = $1 AND lang = $2",
            article.0,
            lang
        )
        .fetch_optional(&*self.pool)
        .await?
        .map(Variant::try_from)
        .transpose()
    }

    pub async fn list_variants(&self, article: ArticleId) -> Result<Vec<Variant>, BrainError> {
        sqlx::query_as!(
            VariantRow,
            "SELECT article_id, lang, title, description, body, front_matter, properties,
                    content_hash, revision, updated_by, created_at, updated_at
               FROM dynbrain_article_variant WHERE article_id = $1 ORDER BY lang",
            article.0
        )
        .fetch_all(&*self.pool)
        .await?
        .into_iter()
        .map(Variant::try_from)
        .collect()
    }

    pub async fn list_revisions(
        &self,
        article: ArticleId,
        lang: &str,
    ) -> Result<Vec<Revision>, BrainError> {
        sqlx::query_as!(
            RevisionRow,
            "SELECT article_id, lang, revision, title, description, body, front_matter, properties,
                    content_hash, author_id, created_at
               FROM dynbrain_article_revision WHERE article_id = $1 AND lang = $2
              ORDER BY revision",
            article.0,
            lang
        )
        .fetch_all(&*self.pool)
        .await?
        .into_iter()
        .map(Revision::try_from)
        .collect()
    }
}
