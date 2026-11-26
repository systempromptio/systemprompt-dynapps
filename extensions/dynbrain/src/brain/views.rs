//! What an operation answers. The field names are the ones the `docs_*` tools
//! answer with; what Governance adds (`slug`, `inherit_access`, `level`) is a
//! new field, never a renamed one.

use chrono::{DateTime, Utc};
use serde::Serialize;
use systemprompt::identifiers::UserId;

use super::args::Object;
use crate::access::{AccessEntry, Grant, Level, SubjectKind};
use crate::ids::{ArticleId, SpaceId};
use crate::properties::PropertyDefinition;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VariantInfo {
    pub lang: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpaceSummary {
    pub space: SpaceId,
    pub slug: String,
    pub name: String,
    pub lang: String,
    pub langs: Vec<String>,
    pub article_count: usize,
    pub level: Level,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpaceList {
    pub spaces: Vec<SpaceSummary>,
    pub total: usize,
    pub shown: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChildSummary {
    pub article_id: ArticleId,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArticleView {
    pub found: bool,
    pub article_id: ArticleId,
    pub space: SpaceId,
    pub path: String,
    pub title: String,
    pub description: String,
    pub lang: String,
    pub variant: VariantInfo,
    pub markdown: String,
    pub front_matter: Object,
    pub metadata: Object,
    pub revision: i32,
    pub inherit_access: bool,
    pub can_write: bool,
    pub children: Vec<ChildSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ListedArticle {
    pub article_id: ArticleId,
    pub path: String,
    pub title: String,
    pub lang: String,
    pub parent_id: Option<ArticleId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArticleList {
    pub documents: Vec<ListedArticle>,
    pub total: usize,
    pub shown: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Anchor {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OutlineNode {
    pub id: ArticleId,
    pub title: String,
    pub lang: String,
    pub depth: u32,
    pub parent_id: Option<ArticleId>,
    pub has_children: bool,
    pub child_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Outline {
    pub anchor: Anchor,
    pub nodes: Vec<OutlineNode>,
    pub total: usize,
    pub shown: usize,
    pub truncated: bool,
    pub has_more: bool,
    pub offset: usize,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Export {
    pub space: SpaceId,
    pub format: String,
    pub mimetype: &'static str,
    pub markdown: String,
    pub size_bytes: usize,
    pub article_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MetadataItem {
    pub id: ArticleId,
    pub title: String,
    pub description: String,
    pub path: String,
    pub lang: String,
    pub space: SpaceId,
    pub is_synced: bool,
    pub can_write: bool,
    pub inherit_access: bool,
    pub author: Option<UserId>,
    pub created: DateTime<Utc>,
    pub last_editor: Option<UserId>,
    pub updated: DateTime<Utc>,
    pub parent_id: Option<ArticleId>,
    pub child_count: usize,
    pub variant_langs: Vec<String>,
    pub revision: i32,
    pub front_matter: Object,
    pub metadata: Object,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MetadataList {
    pub documents: Vec<MetadataItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MetadataSchema {
    pub space: SpaceId,
    pub fields: Vec<PropertyDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Created {
    pub article_id: ArticleId,
    pub space: SpaceId,
    pub parent_id: Option<ArticleId>,
    pub title: String,
    pub path: String,
    pub variant: VariantInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatedNode {
    pub title: String,
    pub article_id: ArticleId,
    pub path: String,
    pub children: Vec<Self>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TreeCreated {
    pub parent_id: Option<ArticleId>,
    pub nodes: Vec<CreatedNode>,
    pub created: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Updated {
    pub article_id: ArticleId,
    pub title: String,
    #[serde(rename = "updated")]
    pub applied: bool,
    pub revision: i32,
    pub variant: VariantInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Moved {
    pub article_id: ArticleId,
    pub new_parent_id: Option<ArticleId>,
    #[serde(rename = "moved")]
    pub applied: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Trashed {
    pub article_id: ArticleId,
    #[serde(rename = "trashed")]
    pub applied: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GrantView {
    pub kind: SubjectKind,
    pub id: String,
    pub level: Level,
}

impl From<Grant> for GrantView {
    fn from(grant: Grant) -> Self {
        Self {
            kind: grant.subject.kind,
            id: grant.subject.id,
            level: grant.level,
        }
    }
}

impl From<AccessEntry> for GrantView {
    fn from(entry: AccessEntry) -> Self {
        Self {
            kind: entry.subject.kind,
            id: entry.subject.id,
            level: entry.level,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Grants {
    #[serde(rename = "grants")]
    pub own: Vec<GrantView>,
    pub access: Vec<GrantView>,
    pub inherit_access: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct InheritChanged {
    pub article_id: ArticleId,
    pub inherit_access: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SearchHit {
    pub article_id: ArticleId,
    pub path: String,
    pub title: String,
    pub lang: String,
    pub snippet: String,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SearchResults {
    pub results: Vec<SearchHit>,
    pub confidence: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Answer {
    pub question: String,
    pub context: String,
    pub sources: Vec<SearchHit>,
    pub confidence: Option<String>,
}
