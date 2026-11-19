//! The shapes a tool exchanges with the file layer: the upload request and
//! its signed ticket, a downloaded file to keep, the [`FileOut`] a tool
//! hands back, and a [`ResolvedFile`] opened for reading.

use std::fmt;

use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use systemprompt::identifiers::DocumentUploadId;
use systemprompt::traits::{FileByteStream, FileStorageError};

use super::error::FilesError;
use super::refs::FileRef;

/// The file a client is about to send through a signed upload.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UploadRequest {
    #[schemars(description = "File name, without any path.")]
    #[serde(default)]
    pub name: String,
    #[schemars(description = "MIME type, e.g. application/pdf.")]
    #[serde(default)]
    pub mime: String,
    #[schemars(description = "Exact size of the file in bytes.")]
    #[serde(default)]
    pub size: u64,
    #[schemars(description = "SHA-256 of the file as 64 hexadecimal characters.")]
    #[serde(default)]
    pub sha256: String,
    #[schemars(
        description = "Set only to renew the token of an unfinished upload; the other fields are then ignored."
    )]
    #[serde(default)]
    pub upload_id: String,
}

/// Where a client sends a file, and the reference that names it once the
/// upload is complete. Its `Debug` leaves the upload token out.
#[derive(Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct UploadTicket {
    pub file_ref: FileRef,
    pub upload_id: DocumentUploadId,
    pub part_size: u64,
    pub part_count: u32,
    pub parts_url: String,
    pub status_url: String,
    pub complete_url: String,
    pub upload_token: String,
    pub token_header: String,
    #[schemars(with = "String")]
    pub expires_at: DateTime<Utc>,
    pub instructions: String,
}

impl fmt::Debug for UploadTicket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UploadTicket")
            .field("upload_id", &self.upload_id)
            .field("part_size", &self.part_size)
            .field("part_count", &self.part_count)
            .field("expires_at", &self.expires_at)
            .finish_non_exhaustive()
    }
}

/// Bytes a server fetched from its upstream, to keep as a document of the
/// caller instead of returning them to the model.
#[derive(Clone, PartialEq, Eq)]
pub struct DownloadedFile {
    pub name: String,
    pub mime: String,
    pub bytes: Vec<u8>,
}

impl fmt::Debug for DownloadedFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DownloadedFile")
            .field("mime", &self.mime)
            .field("size", &self.bytes.len())
            .finish_non_exhaustive()
    }
}

/// A file a tool hands back: a reference to the caller's document and,
/// when the API signed one, a short-lived link to its bytes. Its `Debug`
/// leaves the signed link out.
#[derive(Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct FileOut {
    pub file_ref: FileRef,
    pub name: String,
    pub mime: String,
    pub size_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<String>")]
    pub expires_at: Option<DateTime<Utc>>,
}

impl fmt::Debug for FileOut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileOut")
            .field("file_ref", &self.file_ref)
            .field("mime", &self.mime)
            .field("size_bytes", &self.size_bytes)
            .field("expires_at", &self.expires_at)
            .finish_non_exhaustive()
    }
}

/// A referenced file, checked and opened for reading as the caller.
///
/// The body is a stream capped at the server's size limit: both
/// [`ResolvedFile::into_stream`] and [`ResolvedFile::read_all`] stop with an
/// error at the first byte past it, whatever size the store reported.
pub struct ResolvedFile {
    pub file_ref: FileRef,
    pub name: String,
    pub mime: String,
    pub size_bytes: Option<u64>,
    max_bytes: u64,
    body: FileByteStream,
}

impl fmt::Debug for ResolvedFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResolvedFile")
            .field("file_ref", &self.file_ref)
            .field("mime", &self.mime)
            .field("size_bytes", &self.size_bytes)
            .finish_non_exhaustive()
    }
}

/// The parts a [`ResolvedFile`] is assembled from.
pub struct ResolvedParts {
    pub file_ref: FileRef,
    pub name: String,
    pub mime: String,
    pub size_bytes: Option<u64>,
    pub max_bytes: u64,
    pub body: FileByteStream,
}

impl fmt::Debug for ResolvedParts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResolvedParts")
            .field("file_ref", &self.file_ref)
            .finish_non_exhaustive()
    }
}

impl ResolvedFile {
    #[must_use]
    pub fn new(parts: ResolvedParts) -> Self {
        Self {
            file_ref: parts.file_ref,
            name: parts.name,
            mime: parts.mime,
            size_bytes: parts.size_bytes,
            max_bytes: parts.max_bytes,
            body: parts.body,
        }
    }

    #[must_use]
    pub fn into_stream(self) -> FileByteStream {
        let limit = self.max_bytes;
        let mut seen: u64 = 0;
        Box::pin(self.body.map(move |chunk| {
            let chunk = chunk?;
            seen = seen.saturating_add(chunk.len() as u64);
            if seen > limit {
                return Err(FileStorageError::Validation(format!(
                    "the file passed the {limit}-byte limit"
                )));
            }
            Ok(chunk)
        }))
    }

    pub async fn read_all(self) -> Result<Vec<u8>, FilesError> {
        read_capped(self.body, self.size_bytes, self.max_bytes).await
    }
}

pub(super) async fn read_capped(
    mut body: FileByteStream,
    size_hint: Option<u64>,
    limit: u64,
) -> Result<Vec<u8>, FilesError> {
    let capacity = size_hint.unwrap_or(0).min(limit);
    let mut bytes = Vec::with_capacity(usize::try_from(capacity).unwrap_or(0));
    while let Some(chunk) = body.next().await {
        let chunk = chunk.map_err(|error| {
            tracing::error!(error = %error, "reading a stored file failed");
            FilesError::Unavailable
        })?;
        let size = (bytes.len() as u64).saturating_add(chunk.len() as u64);
        if size > limit {
            return Err(FilesError::TooLarge { size, limit });
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
