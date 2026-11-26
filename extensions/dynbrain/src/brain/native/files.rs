//! The file operations of the `docs_*` tools: a one-time upload URL, taking
//! the bytes it receives, adopting them as a file of an article, listing an
//! article's files and a signed download link for one of them.
//!
//! Uploading and adopting need `write` on the article; listing and linking
//! need `read`. A file the person cannot read, of a trashed article, or in
//! the trash is `not_found`, exactly as one that does not exist. An upload
//! slot that is unknown, already used or another person's is refused with
//! one same answer.

use std::path::Path;

use chrono::Utc;
use sha2::{Digest, Sha256};
use systemprompt::identifiers::UserId;

use super::NativeBrain;
use super::read::page;
use super::serve::download_url;
use super::shape::require;
use crate::access::Level;
use crate::assets::link::{DOWNLOAD_LINK_TTL, LinkPurpose, SignedLink, UPLOAD_LINK_TTL};
use crate::assets::markdown::reference;
use crate::assets::model::{Asset, NewAsset, StoredUpload};
use crate::assets::{STORAGE_PREFIX, sniff};
use crate::brain::Reply;
use crate::brain::assets::{
    AssetAttached, AssetEntry, AssetLink, AssetLinkArgs, AssetList, AssetListArgs, AssetUploadArgs,
    Received, RequestUploadArgs, UploadTicket,
};
use crate::brain::fault::BrainFault;
use crate::error::BrainError;
use crate::ids::{ArticleId, UploadId};
use crate::model::Article;
use crate::router::EXTENSION_BASE_PATH;

pub(super) const UNSIGNED: BrainFault = BrainFault::Unavailable("file links cannot be signed");
const UPLOAD_REFUSED: &str =
    "the upload_id is unknown, already used, not yours, or received no file";
const INSTRUCTIONS: &str = "POST the file to upload_url as the raw request body (for example `curl --data-binary @file`), before expires_at. The answer has the upload_id; then call asset_upload with it, the article and the filename.";

impl NativeBrain {
    pub(super) async fn issue_upload(
        &self,
        user: &UserId,
        args: &RequestUploadArgs,
    ) -> Reply<UploadTicket> {
        let files = self.files()?;
        let article = self.resolve(args.article_id, args.path.as_deref()).await?;
        self.writable(article, user).await?;
        let upload = UploadId::generate();
        let expires_at = Utc::now() + UPLOAD_LINK_TTL;
        let link = files
            .signer
            .sign(LinkPurpose::Upload, upload.as_str(), expires_at)
            .ok_or(UNSIGNED)?;
        self.repo.open_upload(&upload, user, expires_at).await?;
        tracing::info!(%upload, %article, %user, "dynbrain: upload link issued");
        Ok(UploadTicket {
            upload_url: format!(
                "{}{EXTENSION_BASE_PATH}/uploads/{upload}?{}",
                files.base_url.trim_end_matches('/'),
                link.query()
            ),
            upload_id: upload,
            method: "POST",
            max_bytes: files.max_upload_bytes,
            expires_at,
            expires_minutes: UPLOAD_LINK_TTL.num_minutes(),
            article_id: article,
            filename: args.filename.clone(),
            instructions: INSTRUCTIONS,
        })
    }

    pub(super) async fn take_upload(
        &self,
        upload: &UploadId,
        link: &SignedLink,
        bytes: &[u8],
    ) -> Reply<Received> {
        let files = self.files()?;
        if !files
            .signer
            .verify(LinkPurpose::Upload, upload.as_str(), link, Utc::now())
        {
            return Err(BrainFault::NotFound);
        }
        if bytes.is_empty() {
            return Err(BrainFault::Invalid("the file is empty"));
        }
        if bytes.len() as u64 > files.max_upload_bytes {
            return Err(BrainFault::TooLarge(files.max_upload_bytes));
        }
        if !self.repo.claim_upload(upload).await? {
            return Err(BrainFault::NotFound);
        }
        let key = format!("{STORAGE_PREFIX}/uploads/{upload}");
        let stored = files
            .storage
            .store(Path::new(&key), bytes)
            .await
            .map_err(BrainError::from)?;
        let kept = StoredUpload {
            storage_ref: stored.0,
            size_bytes: bytes.len() as i64,
            sha256: hex::encode(Sha256::digest(bytes)),
            mimetype: sniff::detect(bytes).to_owned(),
        };
        self.repo.fill_upload(upload, &kept).await?;
        tracing::info!(%upload, size = bytes.len(), "dynbrain: upload received");
        Ok(Received {
            upload_id: upload.clone(),
            size_bytes: bytes.len() as u64,
            mimetype: kept.mimetype,
            sha256: kept.sha256,
        })
    }

    pub(super) async fn adopt_upload(
        &self,
        user: &UserId,
        args: &AssetUploadArgs,
    ) -> Reply<AssetAttached> {
        self.files()?;
        let id = self.resolve(args.article_id, args.path.as_deref()).await?;
        let article = self.writable(id, user).await?;
        let upload = args.upload_id.as_ref().ok_or(BrainFault::Invalid(
            "pass the upload_id the upload answered with",
        ))?;
        let name = clean_name(args.filename.as_deref()).ok_or(BrainFault::Invalid(
            "pass a filename of 1 to 255 characters",
        ))?;
        if let Some(lang) = &args.lang {
            let space = self.space_row(article.space).await?;
            if !space.langs.contains(lang) {
                return Err(BrainFault::Refused(BrainError::LanguageNotInSpace {
                    lang: lang.clone(),
                    space: space.id,
                }));
            }
        }
        let new = NewAsset {
            upload,
            owner: user,
            article: id,
            lang: args.lang.as_deref(),
            name: &name,
        };
        let asset = self
            .repo
            .attach_upload(&new)
            .await?
            .ok_or(BrainFault::Invalid(UPLOAD_REFUSED))?;
        tracing::info!(asset = %asset.id, article = %id, %user, "dynbrain: file attached");
        Ok(AssetAttached {
            attached: true,
            article_id: id,
            file: entry(&asset),
        })
    }

    pub(super) async fn files_of(&self, user: &UserId, args: &AssetListArgs) -> Reply<AssetList> {
        let id = self.resolve(args.article_id, args.path.as_deref()).await?;
        self.reach(id, user).await?;
        let wanted = args.lang.as_deref();
        let listed: Vec<AssetEntry> = self
            .repo
            .list_assets(id)
            .await?
            .iter()
            .filter(|asset| asset.trashed_at.is_none())
            .filter(|asset| !matches!((wanted, asset.lang.as_deref()), (Some(wanted), Some(own)) if wanted != own))
            .map(entry)
            .collect();
        let (total, offset, files) = page(listed, args.offset, args.limit, 50, 200);
        Ok(AssetList {
            article_id: id,
            total,
            shown: files.len(),
            offset,
            files,
        })
    }

    pub(super) async fn link_for(&self, user: &UserId, args: &AssetLinkArgs) -> Reply<AssetLink> {
        let files = self.files()?;
        let id = args
            .attachment_id
            .ok_or(BrainFault::Invalid("pass the attachment_id of the file"))?;
        let asset = self.readable_asset(id, user).await?;
        if args.article_id.is_some_and(|shown| shown != asset.article) {
            return Err(BrainFault::NotFound);
        }
        let (download_url, expires_at) = download_url(files, id)?;
        tracing::info!(asset = %id, %user, "dynbrain: download link issued");
        Ok(AssetLink {
            found: true,
            attachment_id: id,
            article_id: asset.article,
            name: asset.name,
            mimetype: asset.mimetype,
            size_bytes: asset.size_bytes as u64,
            download_url,
            expires_at,
            expires_minutes: DOWNLOAD_LINK_TTL.num_minutes(),
        })
    }

    async fn writable(&self, article: ArticleId, user: &UserId) -> Reply<Article> {
        let (found, level) = self.reach(article, user).await?;
        require(level, Level::Write)?;
        Ok(found)
    }
}

pub(super) fn entry(asset: &Asset) -> AssetEntry {
    let reference = reference(asset.id);
    let markdown = if sniff::is_inline(&asset.mimetype) {
        format!("![{}]({reference})", asset.name)
    } else {
        format!("[{}]({reference})", asset.name)
    };
    AssetEntry {
        attachment_id: asset.id,
        name: asset.name.clone(),
        mimetype: asset.mimetype.clone(),
        size_bytes: asset.size_bytes as u64,
        lang: asset.lang.clone(),
        reference,
        markdown,
    }
}

fn clean_name(name: Option<&str>) -> Option<String> {
    let last = name?.trim().rsplit(['/', '\\']).next()?.trim();
    let clean: String = last.chars().filter(|c| !c.is_control()).collect();
    let length = clean.chars().count();
    (1..=255).contains(&length).then_some(clean)
}
