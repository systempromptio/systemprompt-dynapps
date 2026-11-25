//! Spaces: create, read, list and edit.

use super::BrainRepository;
use super::types::SpaceRow;
use crate::error::BrainError;
use crate::ids::SpaceId;
use crate::language::is_language_code;
use crate::model::{NewSpace, Space, SpaceEdit};

impl BrainRepository {
    pub async fn insert_space(&self, space: &NewSpace) -> Result<Space, BrainError> {
        if !space.langs.iter().all(|lang| is_language_code(lang)) {
            return Err(BrainError::Invalid(
                "a space language is not a language code",
            ));
        }
        if !space.langs.contains(&space.default_lang) {
            return Err(BrainError::Invalid(
                "the default language is not one of the space's languages",
            ));
        }
        let schema = space.properties.to_document()?;
        let row = sqlx::query_as!(
            SpaceRow,
            "INSERT INTO dynbrain_space (slug, name, default_lang, langs, property_schema, created_by)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, slug, name, default_lang, langs, property_schema, created_by, created_at,
                       updated_at",
            space.slug,
            space.name,
            space.default_lang,
            space.langs.as_slice(),
            schema,
            space.created_by.as_str()
        )
        .fetch_one(&*self.pool)
        .await?;
        Space::try_from(row)
    }

    pub async fn find_space(&self, space: SpaceId) -> Result<Option<Space>, BrainError> {
        sqlx::query_as!(
            SpaceRow,
            "SELECT id, slug, name, default_lang, langs, property_schema, created_by, created_at,
                    updated_at
               FROM dynbrain_space WHERE id = $1",
            space.0
        )
        .fetch_optional(&*self.pool)
        .await?
        .map(Space::try_from)
        .transpose()
    }

    pub async fn list_spaces(&self) -> Result<Vec<Space>, BrainError> {
        sqlx::query_as!(
            SpaceRow,
            "SELECT id, slug, name, default_lang, langs, property_schema, created_by, created_at,
                    updated_at
               FROM dynbrain_space ORDER BY name, id"
        )
        .fetch_all(&*self.pool)
        .await?
        .into_iter()
        .map(Space::try_from)
        .collect()
    }

    pub async fn update_space(
        &self,
        space: SpaceId,
        edit: &SpaceEdit,
    ) -> Result<Option<Space>, BrainError> {
        let schema = edit.properties.to_document()?;
        sqlx::query_as!(
            SpaceRow,
            "UPDATE dynbrain_space SET name = $2, property_schema = $3, updated_at = CURRENT_TIMESTAMP
              WHERE id = $1
             RETURNING id, slug, name, default_lang, langs, property_schema, created_by, created_at,
                       updated_at",
            space.0,
            edit.name,
            schema
        )
        .fetch_optional(&*self.pool)
        .await?
        .map(Space::try_from)
        .transpose()
    }
}
