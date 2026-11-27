//! Serving files: the signed URLs `get` and `export` add beside the
//! Markdown, opening a file for its reader or for the holder of a signed
//! link, and deleting the bytes of a purged article.
//!
//! A signed link is a bearer permission: it carries no person, lasts ten
//! minutes, and is checked again on use against the file and its article, so
//! a file trashed after its link was issued is no longer served. Every
//! signed download is logged. `purge_assets` is what purging an article
//! does to its files: it deletes their rows and their bytes.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use systemprompt::identifiers::UserId;
use systemprompt::traits::{FileStorageError, StoredFileId};

use super::NativeBrain;
use super::files::UNSIGNED;
use crate::assets::AssetFiles;
use crate::assets::link::{DOWNLOAD_LINK_TTL, LinkPurpose, SignedLink};
use crate::assets::markdown::{reference, references};
use crate::assets::model::Asset;
use crate::brain::Reply;
use crate::brain::assets::AssetFile;
use crate::brain::fault::BrainFault;
use crate::error::BrainError;
use crate::ids::{ArticleId, AssetId};
use crate::router::EXTENSION_BASE_PATH;

impl NativeBrain {
    pub(super) async fn asset_urls(
        &self,
        user: &UserId,
        texts: &[&str],
    ) -> Reply<BTreeMap<String, String>> {
        let mut urls = BTreeMap::new();
        let Some(files) = &self.files else {
            return Ok(urls);
        };
        for text in texts {
            for id in references(text) {
                if urls.contains_key(&reference(id)) {
                    continue;
                }
                match self.readable_asset(id, user).await {
                    Ok(_) => {
                        let (url, _) = download_url(files, id)?;
                        urls.insert(reference(id), url);
                    },
                    Err(BrainFault::NotFound) => {},
                    Err(other) => return Err(other),
                }
            }
        }
        Ok(urls)
    }

    pub(super) async fn readable_asset(&self, id: AssetId, user: &UserId) -> Reply<Asset> {
        let asset = self
            .repo
            .find_asset(id)
            .await?
            .filter(|asset| asset.trashed_at.is_none())
            .ok_or(BrainFault::NotFound)?;
        self.reach(asset.article, user).await?;
        Ok(asset)
    }

    pub(super) async fn open_readable(&self, user: &UserId, id: AssetId) -> Reply<AssetFile> {
        let files = self.files()?;
        let asset = self.readable_asset(id, user).await?;
        stream(files, asset).await
    }

    pub(super) async fn open_linked(&self, id: AssetId, link: &SignedLink) -> Reply<AssetFile> {
        let files = self.files()?;
        if !files
            .signer
            .verify(LinkPurpose::Download, &id.to_string(), link, Utc::now())
        {
            return Err(BrainFault::NotFound);
        }
        let asset = self
            .repo
            .find_asset(id)
            .await?
            .filter(|asset| asset.trashed_at.is_none())
            .ok_or(BrainFault::NotFound)?;
        self.repo
            .find_article(asset.article)
            .await?
            .filter(|article| article.trashed_at.is_none())
            .ok_or(BrainFault::NotFound)?;
        tracing::info!(asset = %id, article = %asset.article, expires = link.exp, "dynbrain: signed download");
        stream(files, asset).await
    }

    pub async fn purge_assets(&self, article: ArticleId) -> Reply<usize> {
        let files = self.files()?;
        let stored = self.repo.delete_assets(article).await?;
        for key in &stored {
            if let Err(error) = files.storage.delete(&StoredFileId::new(key.as_str())).await {
                tracing::warn!(%error, %article, "dynbrain: the bytes of a purged file were not deleted");
            }
        }
        Ok(stored.len())
    }
}

pub(super) fn download_url(files: &AssetFiles, id: AssetId) -> Reply<(String, DateTime<Utc>)> {
    let expires_at = Utc::now() + DOWNLOAD_LINK_TTL;
    let link = files
        .signer
        .sign(LinkPurpose::Download, &id.to_string(), expires_at)
        .ok_or(UNSIGNED)?;
    let url = format!(
        "{}{EXTENSION_BASE_PATH}/assets/{id}/raw?{}",
        files.base_url.trim_end_matches('/'),
        link.query()
    );
    Ok((url, expires_at))
}

async fn stream(files: &AssetFiles, asset: Asset) -> Reply<AssetFile> {
    let read = files
        .storage
        .open(&StoredFileId::new(asset.storage_ref), None)
        .await
        .map_err(|error| match error {
            FileStorageError::NotFound(_) => BrainFault::NotFound,
            other => BrainFault::Internal(BrainError::Storage(other)),
        })?;
    Ok(AssetFile {
        name: asset.name,
        mimetype: asset.mimetype,
        size_bytes: asset.size_bytes as u64,
        body: read.body,
    })
}
