//! Upload slots and the files of articles.
//!
//! A slot is claimed by its one POST before the bytes are written, so two
//! racing POSTs never both write; it is consumed by its one adoption, in the
//! same transaction that creates the file row, and only by the person it was
//! issued to. The rows hold the storage reference, never the bytes.
//!
//! `claim_upload` answers false for a slot that is unknown, expired or
//! already used; `attach_upload` answers `None`, and changes nothing, for a
//! slot that is not the owner's, was used, or holds no bytes. `list_assets`
//! includes trashed files, and `delete_assets` answers the storage references
//! whose bytes the caller deletes next.

use chrono::{DateTime, Utc};
use systemprompt::identifiers::UserId;

use super::BrainRepository;
use crate::assets::model::{Asset, NewAsset, StoredUpload};
use crate::assets::sniff;
use crate::error::BrainError;
use crate::ids::{ArticleId, AssetId, UploadId};

struct AssetRow {
    id: i64,
    article_id: i64,
    lang: Option<String>,
    name: String,
    mimetype: String,
    size_bytes: i64,
    sha256: String,
    storage_ref: String,
    created_by: Option<String>,
    created_at: DateTime<Utc>,
    trashed_at: Option<DateTime<Utc>>,
}

impl From<AssetRow> for Asset {
    fn from(row: AssetRow) -> Self {
        Self {
            id: AssetId(row.id),
            article: ArticleId(row.article_id),
            lang: row.lang,
            name: row.name,
            mimetype: row.mimetype,
            size_bytes: row.size_bytes,
            sha256: row.sha256,
            storage_ref: row.storage_ref,
            created_by: row.created_by.map(UserId::new),
            created_at: row.created_at,
            trashed_at: row.trashed_at,
        }
    }
}

impl BrainRepository {
    pub async fn open_upload(
        &self,
        upload: &UploadId,
        owner: &UserId,
        expires_at: DateTime<Utc>,
    ) -> Result<(), BrainError> {
        sqlx::query!(
            "INSERT INTO dynbrain_upload (id, user_id, expires_at) VALUES ($1, $2, $3)",
            upload.as_str(),
            owner.as_str(),
            expires_at
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    pub async fn claim_upload(&self, upload: &UploadId) -> Result<bool, BrainError> {
        let done = sqlx::query!(
            "UPDATE dynbrain_upload SET received_at = CURRENT_TIMESTAMP WHERE id = $1 AND received_at IS NULL AND expires_at > CURRENT_TIMESTAMP",
            upload.as_str()
        )
        .execute(&*self.pool)
        .await?;
        Ok(done.rows_affected() == 1)
    }

    pub async fn fill_upload(
        &self,
        upload: &UploadId,
        stored: &StoredUpload,
    ) -> Result<(), BrainError> {
        sqlx::query!(
            "UPDATE dynbrain_upload SET storage_ref = $2, size_bytes = $3, sha256 = $4, mimetype = $5 WHERE id = $1",
            upload.as_str(),
            stored.storage_ref,
            stored.size_bytes,
            stored.sha256,
            stored.mimetype
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    pub async fn attach_upload(&self, new: &NewAsset<'_>) -> Result<Option<Asset>, BrainError> {
        let mut tx = self.pool.begin().await?;
        let Some(taken) = sqlx::query!(
            "UPDATE dynbrain_upload SET consumed_at = CURRENT_TIMESTAMP WHERE id = $1 AND user_id = $2 AND consumed_at IS NULL AND storage_ref IS NOT NULL RETURNING storage_ref, size_bytes, sha256, mimetype",
            new.upload.as_str(),
            new.owner.as_str()
        )
        .fetch_optional(&mut *tx)
        .await?
        else {
            return Ok(None);
        };
        let (Some(storage_ref), Some(size_bytes), Some(sha256), Some(detected)) = (
            taken.storage_ref,
            taken.size_bytes,
            taken.sha256,
            taken.mimetype,
        ) else {
            return Ok(None);
        };
        let row = sqlx::query_as!(
            AssetRow,
            "INSERT INTO dynbrain_asset (article_id, lang, name, mimetype, size_bytes, sha256, storage_ref, upload_id, created_by) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id, article_id, lang, name, mimetype, size_bytes, sha256, storage_ref, created_by, created_at, trashed_at",
            new.article.0,
            new.lang,
            new.name,
            sniff::refine(&detected, new.name),
            size_bytes,
            sha256,
            storage_ref,
            new.upload.as_str(),
            new.owner.as_str()
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(Some(Asset::from(row)))
    }

    pub async fn list_assets(&self, article: ArticleId) -> Result<Vec<Asset>, BrainError> {
        let rows = sqlx::query_as!(
            AssetRow,
            "SELECT id, article_id, lang, name, mimetype, size_bytes, sha256, storage_ref, created_by, created_at, trashed_at FROM dynbrain_asset WHERE article_id = $1 ORDER BY id",
            article.0
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows.into_iter().map(Asset::from).collect())
    }

    pub async fn find_asset(&self, asset: AssetId) -> Result<Option<Asset>, BrainError> {
        let row = sqlx::query_as!(
            AssetRow,
            "SELECT id, article_id, lang, name, mimetype, size_bytes, sha256, storage_ref, created_by, created_at, trashed_at FROM dynbrain_asset WHERE id = $1",
            asset.0
        )
        .fetch_optional(&*self.pool)
        .await?;
        Ok(row.map(Asset::from))
    }

    pub async fn delete_assets(&self, article: ArticleId) -> Result<Vec<String>, BrainError> {
        let rows = sqlx::query!(
            "DELETE FROM dynbrain_asset WHERE article_id = $1 RETURNING storage_ref",
            article.0
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows.into_iter().map(|row| row.storage_ref).collect())
    }
}
