//! What the file operations take and answer.
//!
//! The names follow the `docs_*` file tools (`attachment_id`, `filename`,
//! `files`, `download_url`, `expires_minutes`); `ref` is the stable Markdown
//! reference, `asset://<id>`, which Governance adds. [`AssetFile`] is the
//! open file the download routes stream.

use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use systemprompt::traits::FileByteStream;

use crate::ids::{ArticleId, AssetId, UploadId};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct RequestUploadArgs {
    pub article_id: Option<ArticleId>,
    pub path: Option<String>,
    pub filename: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AssetUploadArgs {
    pub article_id: Option<ArticleId>,
    pub path: Option<String>,
    pub upload_id: Option<UploadId>,
    #[serde(alias = "name")]
    pub filename: Option<String>,
    pub lang: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AssetListArgs {
    pub article_id: Option<ArticleId>,
    pub path: Option<String>,
    pub lang: Option<String>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(default)]
pub struct AssetLinkArgs {
    pub attachment_id: Option<AssetId>,
    pub article_id: Option<ArticleId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UploadTicket {
    pub upload_id: UploadId,
    pub upload_url: String,
    pub method: &'static str,
    pub max_bytes: u64,
    pub expires_at: DateTime<Utc>,
    pub expires_minutes: i64,
    pub article_id: ArticleId,
    pub filename: Option<String>,
    pub instructions: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Received {
    pub upload_id: UploadId,
    pub size_bytes: u64,
    pub mimetype: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AssetEntry {
    pub attachment_id: AssetId,
    pub name: String,
    pub mimetype: String,
    pub size_bytes: u64,
    pub lang: Option<String>,
    #[serde(rename = "ref")]
    pub reference: String,
    pub markdown: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AssetAttached {
    pub attached: bool,
    pub article_id: ArticleId,
    #[serde(flatten)]
    pub file: AssetEntry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AssetList {
    pub article_id: ArticleId,
    pub files: Vec<AssetEntry>,
    pub total: usize,
    pub shown: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AssetLink {
    pub found: bool,
    pub attachment_id: AssetId,
    pub article_id: ArticleId,
    pub name: String,
    pub mimetype: String,
    pub size_bytes: u64,
    pub download_url: String,
    pub expires_at: DateTime<Utc>,
    pub expires_minutes: i64,
}

/// An open file: what the download routes need for their headers, and the
/// bytes as they come from the file storage.
pub struct AssetFile {
    pub name: String,
    pub mimetype: String,
    pub size_bytes: u64,
    pub body: FileByteStream,
}

impl fmt::Debug for AssetFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AssetFile")
            .field("name", &self.name)
            .field("mimetype", &self.mimetype)
            .field("size_bytes", &self.size_bytes)
            .finish_non_exhaustive()
    }
}
