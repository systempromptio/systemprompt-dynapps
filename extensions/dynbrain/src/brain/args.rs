//! What an operation takes.
//!
//! The field names are the ones of the `docs_*`
//! tools, so the proxy can pass a request through unchanged and the MCP server
//! can deserialize a tool call straight into these types.
//!
//! Every field is optional on the wire; the operation says which it needs. A
//! REST route that names its target in the path overwrites the matching field
//! with it.

use serde::Deserialize;
// JSON: front matter, typed metadata and `where` filters are free JSON objects,
// as in the `docs_*` tools.
use serde_json::{Map, Value};

use crate::access::{Level, SubjectKind};
use crate::ids::{ArticleId, SpaceId};

pub type Object = Map<String, Value>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    First,
    Last,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct SearchArgs {
    pub query: String,
    pub limit: Option<u32>,
    pub lang: Option<String>,
    pub space: Option<SpaceId>,
    #[serde(rename = "where")]
    pub filter: Option<Object>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AnswerArgs {
    pub question: String,
    pub lang: Option<String>,
    pub space: Option<SpaceId>,
    pub max_sources: Option<u32>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct GetArgs {
    pub article_id: Option<ArticleId>,
    pub path: Option<String>,
    pub lang: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ListSpacesArgs {
    pub query: Option<String>,
    pub lang: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ListArgs {
    pub space: SpaceId,
    pub lang: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    #[serde(rename = "where")]
    pub filter: Option<Object>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct OutlineArgs {
    pub node: Option<ArticleId>,
    pub space: Option<SpaceId>,
    pub mode: Option<String>,
    pub depth: Option<i32>,
    pub max_nodes: Option<u32>,
    pub max_levels: Option<u32>,
    pub lang: Option<String>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ExportArgs {
    pub space: SpaceId,
    pub format: Option<String>,
    pub lang: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct MetadataArgs {
    pub article_id: Option<ArticleId>,
    pub path: Option<String>,
    pub article_ids: Option<Vec<ArticleId>>,
    pub limit: Option<u32>,
    pub lang: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct MetadataSchemaArgs {
    pub article_id: Option<ArticleId>,
    pub path: Option<String>,
    pub space: Option<SpaceId>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CreateArgs {
    pub parent_id: Option<ArticleId>,
    pub space: Option<SpaceId>,
    pub title: String,
    pub markdown: Option<String>,
    pub position: Option<Position>,
    pub front_matter: Option<Object>,
    pub metadata: Option<Object>,
    pub lang: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct NewNode {
    pub title: String,
    pub markdown: Option<String>,
    pub front_matter: Option<Object>,
    pub metadata: Option<Object>,
    pub children: Vec<Self>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct TreeArgs {
    pub parent_id: Option<ArticleId>,
    pub space: Option<SpaceId>,
    pub nodes: Vec<NewNode>,
    pub lang: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct UpdateArgs {
    pub article_id: ArticleId,
    pub title: Option<String>,
    pub markdown: Option<String>,
    pub front_matter: Option<Object>,
    pub metadata: Option<Object>,
    pub lang: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(default)]
pub struct MoveArgs {
    pub article_id: ArticleId,
    pub new_parent_id: Option<ArticleId>,
    pub position: Option<Position>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantTarget {
    Space(SpaceId),
    Article(ArticleId),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GrantInput {
    pub kind: SubjectKind,
    pub id: String,
    pub level: Level,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct SetGrantsArgs {
    pub grants: Vec<GrantInput>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(default)]
pub struct InheritArgs {
    pub inherit: bool,
    pub copy_inherited: bool,
}
