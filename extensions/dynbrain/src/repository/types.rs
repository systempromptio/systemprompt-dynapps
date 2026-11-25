//! Rows as the queries return them, and their conversion into the model.

use chrono::{DateTime, Utc};
// JSON: JSONB columns decode as JSON documents.
use serde_json::{Map, Value};
use systemprompt::identifiers::UserId;

use crate::access::{AccessEntry, Grant, Level, Subject, SubjectKind, TreeNode};
use crate::error::BrainError;
use crate::ids::{ArticleId, SpaceId};
use crate::model::{Article, RagMode, Revision, Space, Variant};
use crate::properties::PropertySchema;

#[derive(Debug)]
pub struct SpaceRow {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub default_lang: String,
    pub langs: Vec<String>,
    pub property_schema: Value,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct ArticleRow {
    pub id: i64,
    pub space_id: i64,
    pub parent_id: Option<i64>,
    pub position: i32,
    pub slug: String,
    pub icon: Option<String>,
    pub rag_mode: String,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub trashed_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct VariantRow {
    pub article_id: i64,
    pub lang: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub front_matter: Value,
    pub properties: Value,
    pub content_hash: String,
    pub revision: i32,
    pub updated_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct RevisionRow {
    pub article_id: i64,
    pub lang: String,
    pub revision: i32,
    pub title: String,
    pub description: String,
    pub body: String,
    pub front_matter: Value,
    pub properties: Value,
    pub content_hash: String,
    pub author_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct GrantRow {
    pub article_id: i64,
    pub subject_kind: String,
    pub subject_id: String,
    pub level: i16,
    pub inherit: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct NodeRow {
    pub id: i64,
    pub parent_id: Option<i64>,
}

/// The space row a tree write locks: its id and its owner.
#[derive(Debug)]
pub struct SpaceLockRow {
    pub id: i64,
    pub created_by: Option<String>,
}

/// What a variant write needs from its article and space.
#[derive(Debug)]
pub struct VariantTargetRow {
    pub space_id: i64,
    pub langs: Vec<String>,
    pub property_schema: Value,
}

impl TryFrom<SpaceRow> for Space {
    type Error = BrainError;

    fn try_from(row: SpaceRow) -> Result<Self, BrainError> {
        Ok(Self {
            id: SpaceId(row.id),
            slug: row.slug,
            name: row.name,
            default_lang: row.default_lang,
            langs: row.langs,
            properties: PropertySchema::parse(&row.property_schema)?,
            created_by: row.created_by.map(UserId::new),
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<ArticleRow> for Article {
    type Error = BrainError;

    fn try_from(row: ArticleRow) -> Result<Self, BrainError> {
        Ok(Self {
            id: ArticleId(row.id),
            space: SpaceId(row.space_id),
            parent: row.parent_id.map(ArticleId),
            position: row.position,
            slug: row.slug,
            icon: row.icon,
            rag_mode: RagMode::parse(&row.rag_mode)
                .ok_or(BrainError::Invalid("a stored rag mode is unknown"))?,
            created_by: row.created_by.map(UserId::new),
            created_at: row.created_at,
            updated_at: row.updated_at,
            trashed_at: row.trashed_at,
        })
    }
}

impl TryFrom<VariantRow> for Variant {
    type Error = BrainError;

    fn try_from(row: VariantRow) -> Result<Self, BrainError> {
        Ok(Self {
            article: ArticleId(row.article_id),
            lang: row.lang,
            title: row.title,
            description: row.description,
            body: row.body,
            front_matter: into_object(row.front_matter)?,
            properties: into_object(row.properties)?,
            content_hash: row.content_hash,
            revision: row.revision,
            updated_by: row.updated_by.map(UserId::new),
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<RevisionRow> for Revision {
    type Error = BrainError;

    fn try_from(row: RevisionRow) -> Result<Self, BrainError> {
        Ok(Self {
            article: ArticleId(row.article_id),
            lang: row.lang,
            number: row.revision,
            title: row.title,
            description: row.description,
            body: row.body,
            front_matter: into_object(row.front_matter)?,
            properties: into_object(row.properties)?,
            content_hash: row.content_hash,
            author: row.author_id.map(UserId::new),
            created_at: row.created_at,
        })
    }
}

impl TryFrom<GrantRow> for Grant {
    type Error = BrainError;

    fn try_from(row: GrantRow) -> Result<Self, BrainError> {
        Ok(Self {
            article: ArticleId(row.article_id),
            subject: Subject {
                kind: SubjectKind::parse(&row.subject_kind)
                    .ok_or(BrainError::Invalid("a stored subject kind is unknown"))?,
                id: row.subject_id,
            },
            level: Level::from_db(row.level)
                .ok_or(BrainError::Invalid("a stored level is unknown"))?,
            inherit: row.inherit,
        })
    }
}

impl From<NodeRow> for TreeNode {
    fn from(row: NodeRow) -> Self {
        Self {
            id: ArticleId(row.id),
            parent: row.parent_id.map(ArticleId),
        }
    }
}

// JSON: front matter and properties are JSONB objects (the schema checks it).
fn into_object(value: Value) -> Result<Map<String, Value>, BrainError> {
    let Value::Object(map) = value else {
        return Err(BrainError::Invalid(
            "a stored document is not a JSON object",
        ));
    };
    Ok(map)
}

#[derive(Debug)]
pub struct AccessRow {
    pub article_id: i64,
    pub subject_kind: String,
    pub subject_id: String,
    pub level: i16,
}

impl TryFrom<AccessRow> for AccessEntry {
    type Error = BrainError;

    fn try_from(row: AccessRow) -> Result<Self, BrainError> {
        Ok(Self {
            article: ArticleId(row.article_id),
            subject: Subject {
                kind: SubjectKind::parse(&row.subject_kind)
                    .ok_or(BrainError::Invalid("a stored subject kind is unknown"))?,
                id: row.subject_id,
            },
            level: Level::from_db(row.level)
                .ok_or(BrainError::Invalid("a stored level is unknown"))?,
        })
    }
}
