//! [`FileStore`]: the port the file layer reaches the document store
//! through, and the plain records it answers with.
//!
//! The store reports what it holds (owner, state, size); it does not decide
//! whether the caller may use it. That decision is taken once, by
//! `McpFiles`, for every store, so a second store cannot weaken it.

use std::fmt;

use async_trait::async_trait;
use axum::body::Bytes;
use chrono::{DateTime, Utc};
use systemprompt::files::uploads::NewUpload;
use systemprompt::identifiers::{
    DocumentId, DocumentRevisionId, DocumentUploadId, JwtToken, McpExecutionId, UserId,
};
use systemprompt::models::execution::context::RequestContext;
use systemprompt::runtime::documents::upload_links::SignedUploadLinks;
use systemprompt::traits::{FileByteStream, FileStorageError};

use super::error::FilesError;
use super::refs::StoredRef;

/// Where a stored file is in its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileState {
    Ready { since: Option<DateTime<Utc>> },
    Pending { expires_at: DateTime<Utc> },
    Expired,
    Gone,
}

/// What the store holds for a reference, before any byte is read.
///
/// `pinned` is the reference with a document's revision resolved, so the
/// bytes opened later are the ones described here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRecord {
    pub owner: UserId,
    pub name: String,
    pub mime: String,
    pub size_bytes: Option<u64>,
    pub state: FileState,
    pub pinned: StoredRef,
}

/// A resumable upload session and the terms a client sends its parts on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadSlot {
    pub upload_id: DocumentUploadId,
    pub owner: UserId,
    pub part_size: u64,
    pub part_count: u32,
    pub expires_at: DateTime<Utc>,
    pub open: bool,
}

/// Bytes to keep as a new document of the caller. Its `Debug` shows the
/// size, never the bytes.
#[derive(Clone, PartialEq, Eq)]
pub struct NewStoredFile {
    pub name: String,
    pub mime: String,
    pub bytes: Vec<u8>,
    pub exec_id: McpExecutionId,
}

/// The document the store kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredFile {
    pub document_id: DocumentId,
    pub revision_id: DocumentRevisionId,
    pub name: String,
    pub mime: String,
    pub size_bytes: u64,
}

/// A signed, short-lived link to one document revision. The URL carries
/// its token, so `Debug` leaves it out.
#[derive(Clone, PartialEq, Eq)]
pub struct SignedDownload {
    pub url: String,
    pub expires_at: DateTime<Utc>,
}

impl fmt::Debug for NewStoredFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NewStoredFile")
            .field("mime", &self.mime)
            .field("size", &self.bytes.len())
            .field("exec_id", &self.exec_id)
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for SignedDownload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SignedDownload")
            .field("expires_at", &self.expires_at)
            .finish_non_exhaustive()
    }
}

/// The document store as the file layer sees it. Every call acts as the
/// request's user; a file the user cannot see answers
/// [`FilesError::NotFound`]. Held as `dyn FileStore` so tests can swap the
/// core document store for an in-memory one.
#[async_trait]
pub trait FileStore: Send + Sync + fmt::Debug {
    async fn open_upload(
        &self,
        ctx: &RequestContext,
        upload: NewUpload,
    ) -> Result<UploadSlot, FilesError>;

    async fn find_upload(
        &self,
        ctx: &RequestContext,
        upload: &DocumentUploadId,
    ) -> Result<UploadSlot, FilesError>;

    async fn sign_upload(
        &self,
        hop: Option<&JwtToken>,
        upload: &DocumentUploadId,
    ) -> Result<SignedUploadLinks, FilesError>;

    async fn describe(
        &self,
        ctx: &RequestContext,
        target: &StoredRef,
    ) -> Result<FileRecord, FilesError>;

    async fn open(
        &self,
        ctx: &RequestContext,
        target: &StoredRef,
        max_bytes: u64,
    ) -> Result<FileByteStream, FilesError>;

    async fn store(
        &self,
        ctx: &RequestContext,
        file: &NewStoredFile,
    ) -> Result<StoredFile, FilesError>;

    async fn sign_download(
        &self,
        hop: Option<&JwtToken>,
        document: &DocumentId,
        revision: &DocumentRevisionId,
    ) -> Result<SignedDownload, FilesError>;
}

#[must_use]
pub fn byte_stream(bytes: Vec<u8>) -> FileByteStream {
    let chunk: Result<Bytes, FileStorageError> = Ok(Bytes::from(bytes));
    Box::pin(futures_util::stream::once(std::future::ready(chunk)))
}
