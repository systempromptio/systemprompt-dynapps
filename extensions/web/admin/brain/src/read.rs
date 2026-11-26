//! The Brain's operations as typed reads, run as the signed-in admin.
//!
//! Every read goes through `Brain::run`, the way the transports run an
//! operation, so a native and a proxy instance answer in the same shape.

use dynbrain::brain::fault::BrainFault;
use dynbrain::brain::operation::Operation;
use dynbrain::console::{self, Console};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use systemprompt::identifiers::{JwtToken, UserId};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct SpaceItem {
    pub space: i64,
    pub slug: String,
    pub name: String,
    pub lang: String,
    pub langs: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Listed {
    pub article_id: i64,
    pub path: String,
    pub title: String,
    pub parent_id: Option<i64>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Document {
    pub found: bool,
    pub article_id: i64,
    #[serde(deserialize_with = "space_of")]
    pub space: Option<i64>,
    pub path: String,
    pub title: String,
    pub description: String,
    pub lang: String,
    pub markdown: String,
    pub revision: i32,
    pub inherit_access: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct FileItem {
    pub name: String,
    pub mimetype: String,
    pub size_bytes: u64,
    pub lang: Option<String>,
}

/// The admin a read runs as, with the session token a proxy instance's
/// source needs to answer as them.
#[derive(Debug, Clone, Copy)]
pub struct Who<'a> {
    pub id: &'a UserId,
    pub token: Option<&'a JwtToken>,
}

/// The Brain and the admin who reads it.
#[derive(Debug, Clone, Copy)]
pub struct Reader<'a> {
    pub console: &'a Console,
    pub admin: &'a UserId,
    pub token: Option<&'a JwtToken>,
}

impl<'a> Reader<'a> {
    pub const fn who(&self) -> Who<'a> {
        Who {
            id: self.admin,
            token: self.token,
        }
    }
}

/// A space with the articles of its tree.
#[derive(Debug, Clone, Default)]
pub struct Space {
    pub item: SpaceItem,
    pub articles: Vec<Listed>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct SpaceList {
    spaces: Vec<SpaceItem>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ArticleList {
    documents: Vec<Listed>,
    total: usize,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct FileList {
    files: Vec<FileItem>,
}

/// Why a read of the Brain failed: the Brain refused it, or answered in a
/// shape these reads do not know.
#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error(transparent)]
    Brain(#[from] BrainFault),
    #[error("the Brain answered in an unexpected shape")]
    Shape(#[from] serde_json::Error),
}

const PAGE: usize = 200;

// JSON: the Brain's operations take and return JSON, which these reads type.
pub async fn call<T: DeserializeOwned>(
    console: &Console,
    who: Who<'_>,
    operation: Operation,
    args: serde_json::Value,
) -> Result<T, ReadError> {
    let context = console::admin_context(who.id, who.token);
    let reply = console.brain.run(who.id, &context, operation, args).await?;
    Ok(serde_json::from_value(reply)?)
}

pub async fn read_spaces(console: &Console, who: Who<'_>) -> Result<Vec<Space>, ReadError> {
    let listed: SpaceList =
        call(console, who, Operation::ListSpaces, serde_json::json!({})).await?;
    let mut spaces = Vec::new();
    for item in listed.spaces {
        let articles = read_articles(console, who, item.space).await?;
        spaces.push(Space { item, articles });
    }
    Ok(spaces)
}

async fn read_articles(
    console: &Console,
    who: Who<'_>,
    space: i64,
) -> Result<Vec<Listed>, ReadError> {
    let mut all: Vec<Listed> = Vec::new();
    loop {
        let args = serde_json::json!({ "space": space, "limit": PAGE, "offset": all.len() });
        let page: ArticleList = call(console, who, Operation::List, args).await?;
        let fetched = page.documents.len();
        all.extend(page.documents);
        if fetched == 0 || all.len() >= page.total {
            return Ok(all);
        }
    }
}

pub async fn read_document(
    console: &Console,
    who: Who<'_>,
    article: i64,
    lang: Option<&str>,
) -> Result<Document, ReadError> {
    let args = serde_json::json!({ "article_id": article, "lang": lang });
    call(console, who, Operation::Get, args).await
}

pub async fn read_files(
    console: &Console,
    who: Who<'_>,
    article: i64,
) -> Result<Vec<FileItem>, ReadError> {
    let args = serde_json::json!({ "article_id": article });
    let listed: FileList = call(console, who, Operation::AssetList, args).await?;
    Ok(listed.files)
}

// JSON: a proxy source names an article outside every space `false`, not a
// number, so anything that is not an id reads as no space.
fn space_of<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<i64>, D::Error> {
    Ok(serde_json::Value::deserialize(deserializer)?.as_i64())
}
