//! What the repository reads and writes: spaces, articles, variants and
//! revisions, and the inputs that create or change them.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
// JSON: front matter and property values are kept as the JSON objects written.
use serde_json::{Map, Value};
use systemprompt::identifiers::UserId;

use crate::ids::{ArticleId, SpaceId};
use crate::properties::PropertySchema;

/// A documentation space.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Space {
    pub id: SpaceId,
    pub slug: String,
    pub name: String,
    pub default_lang: String,
    pub langs: Vec<String>,
    pub properties: PropertySchema,
    pub created_by: Option<UserId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A space to create. `created_by` becomes its owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewSpace {
    pub slug: String,
    pub name: String,
    pub default_lang: String,
    pub langs: Vec<String>,
    pub properties: PropertySchema,
    pub created_by: UserId,
}

/// What may change in a space once it exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceEdit {
    pub name: String,
    pub properties: PropertySchema,
}

/// How an article takes part in the retrieval index: `Default` follows the
/// eligibility rules, `Always` and `Never` force it in or out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RagMode {
    Default,
    Always,
    Never,
}

impl RagMode {
    pub const fn as_db(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Always => "always",
            Self::Never => "never",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "default" => Some(Self::Default),
            "always" => Some(Self::Always),
            "never" => Some(Self::Never),
            _ => None,
        }
    }
}

/// An article of a space's tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Article {
    pub id: ArticleId,
    pub space: SpaceId,
    pub parent: Option<ArticleId>,
    pub position: i32,
    pub slug: String,
    pub icon: Option<String>,
    pub rag_mode: RagMode,
    pub created_by: Option<UserId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub trashed_at: Option<DateTime<Utc>>,
}

/// An article to create. Without a `position` it goes after its siblings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewArticle {
    pub space: SpaceId,
    pub parent: Option<ArticleId>,
    pub position: Option<i32>,
    pub slug: String,
    pub icon: Option<String>,
    pub rag_mode: RagMode,
}

/// Where an article moves: under `parent` (`None` for the top level) of the
/// same space, at `position` or after its new siblings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArticleMove {
    pub parent: Option<ArticleId>,
    pub position: Option<i32>,
}

/// The content of one language of an article.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Variant {
    pub article: ArticleId,
    pub lang: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub front_matter: Map<String, Value>,
    pub properties: Map<String, Value>,
    pub content_hash: String,
    pub revision: i32,
    pub updated_by: Option<UserId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A write of a variant: the whole content, which replaces the previous one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantWrite {
    pub title: String,
    pub description: String,
    pub body: String,
    pub front_matter: Map<String, Value>,
    pub properties: Map<String, Value>,
}

/// One past write of a variant, as it was written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Revision {
    pub article: ArticleId,
    pub lang: String,
    pub revision: i32,
    pub title: String,
    pub description: String,
    pub body: String,
    pub front_matter: Map<String, Value>,
    pub properties: Map<String, Value>,
    pub content_hash: String,
    pub author: Option<UserId>,
    pub created_at: DateTime<Utc>,
}
