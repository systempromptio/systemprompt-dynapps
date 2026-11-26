//! A file of an article as the store keeps it, and what adopting an upload
//! writes.

use chrono::{DateTime, Utc};
use systemprompt::identifiers::UserId;

use crate::ids::{ArticleId, AssetId, UploadId};

/// A file of an article.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    pub id: AssetId,
    pub article: ArticleId,
    pub lang: Option<String>,
    pub name: String,
    pub mimetype: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub storage_ref: String,
    pub created_by: Option<UserId>,
    pub created_at: DateTime<Utc>,
    pub trashed_at: Option<DateTime<Utc>>,
}

/// The bytes an upload slot received, once they are in the file storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredUpload {
    pub storage_ref: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub mimetype: String,
}

/// An upload adopted as a file of an article by the person it was issued to.
#[derive(Debug, Clone, Copy)]
pub struct NewAsset<'a> {
    pub upload: &'a UploadId,
    pub owner: &'a UserId,
    pub article: ArticleId,
    pub lang: Option<&'a str>,
    pub name: &'a str,
}
