//! `resolve_ref`: the one place a [`FileRef`] becomes bytes.
//!
//! Before the store opens anything, the reference must belong to a
//! signed-in caller, name a file the caller owns (a document shared with
//! them is not theirs to send on), be ready and inside its lifetime, and fit
//! the server's limits; the stored type must be one the server admits,
//! whatever type the caller asks to send it as. A file whose size the store
//! does not know is read here, under the size limit, so a refusal for size
//! is audited like any other; a file of known size is handed back as a
//! capped stream. Inline base64 is decoded here under the inline limit.

use chrono::Utc;

use super::audit::FileAction;
use super::error::FilesError;
use super::ops::{Audit, caller, ensure_owner};
use super::outputs::{ResolvedFile, ResolvedParts, read_capped};
use super::policy::{OCTET_STREAM, check_name};
use super::refs::{FileRef, FileSource, StoredRef};
use super::store::{FileRecord, FileState, byte_stream};
use super::{FileCall, McpFiles};

const INLINE_LABEL: &str = "inline";
const INVALID_LABEL: &str = "invalid";
const INLINE_NAME: &str = "attachment";
const UNNAMED_FILE: &str = "file";
const BLOB_NAME_CHARS: usize = 12;

impl McpFiles {
    pub async fn resolve_ref(
        &self,
        call: FileCall<'_>,
        file_ref: &FileRef,
    ) -> Result<ResolvedFile, FilesError> {
        let (label, outcome) = match file_ref.source() {
            Err(error) => (INVALID_LABEL.to_owned(), Err(error)),
            Ok(FileSource::Inline(encoded)) => (
                INLINE_LABEL.to_owned(),
                self.inline(call, file_ref, &encoded),
            ),
            Ok(FileSource::Stored(target)) => (
                target.label(),
                self.open_stored(call, file_ref, &target).await,
            ),
        };
        let size = outcome.as_ref().ok().and_then(|file| file.size_bytes);
        let audit = Audit {
            action: FileAction::Read,
            file: label,
            size_bytes: size,
        };
        self.audited(call, audit, outcome).await
    }

    fn inline(
        &self,
        call: FileCall<'_>,
        file_ref: &FileRef,
        encoded: &str,
    ) -> Result<ResolvedFile, FilesError> {
        caller(call.ctx)?;
        let bytes = self.policy.decode_inline(encoded)?;
        let name = name_or(file_ref, INLINE_NAME)?;
        let mime = self.mime_or(file_ref, OCTET_STREAM)?;
        let size = bytes.len() as u64;
        Ok(ResolvedFile::new(ResolvedParts {
            file_ref: FileRef {
                content_base64: String::new(),
                ..file_ref.clone()
            },
            name,
            mime,
            size_bytes: Some(size),
            max_bytes: self.policy.inline_max_bytes,
            body: byte_stream(bytes),
        }))
    }

    async fn open_stored(
        &self,
        call: FileCall<'_>,
        file_ref: &FileRef,
        target: &StoredRef,
    ) -> Result<ResolvedFile, FilesError> {
        let who = caller(call.ctx)?;
        let record = self.store.describe(call.ctx, target).await?;
        ensure_owner(&who.id, &record.owner, &target.label())?;
        self.ensure_current(&record, target)?;
        if let Some(size) = record.size_bytes {
            self.policy.check_size(size)?;
        }
        let name = name_or(file_ref, &stored_name(&record, target))?;
        let mime = self.mime_or(file_ref, &record.mime)?;
        let max_bytes = self.policy.max_file_bytes;
        let body = self.store.open(call.ctx, &record.pinned, max_bytes).await?;
        let (size_bytes, body) = if let Some(size) = record.size_bytes {
            (Some(size), body)
        } else {
            let bytes = read_capped(body, None, max_bytes).await?;
            self.policy.check_size(bytes.len() as u64)?;
            (Some(bytes.len() as u64), byte_stream(bytes))
        };
        Ok(ResolvedFile::new(ResolvedParts {
            file_ref: pinned_ref(file_ref, &record.pinned),
            name,
            mime,
            size_bytes,
            max_bytes,
            body,
        }))
    }

    fn ensure_current(&self, record: &FileRecord, target: &StoredRef) -> Result<(), FilesError> {
        let now = Utc::now();
        match (record.state, target) {
            (FileState::Ready { since: Some(since) }, _)
                if now.signed_duration_since(since) > self.policy.ref_ttl =>
            {
                Err(self.expired(target.label()))
            },
            (FileState::Ready { .. }, _) => Ok(()),
            (FileState::Pending { expires_at }, StoredRef::Upload(upload)) if expires_at > now => {
                Err(FilesError::NotReady {
                    upload: upload.clone(),
                })
            },
            (FileState::Pending { .. } | FileState::Expired, _) => {
                Err(self.expired(target.label()))
            },
            (FileState::Gone, _) => Err(FilesError::NotFound {
                what: target.label(),
            }),
        }
    }

    fn mime_or(&self, file_ref: &FileRef, stored: &str) -> Result<String, FilesError> {
        let stored = self.policy.download_mime(stored)?;
        if file_ref.mime.trim().is_empty() {
            Ok(stored)
        } else {
            self.policy.mime(&file_ref.mime)
        }
    }
}

fn name_or(file_ref: &FileRef, fallback: &str) -> Result<String, FilesError> {
    if file_ref.name.trim().is_empty() {
        Ok(check_name(fallback).unwrap_or_else(|_unusable| UNNAMED_FILE.to_owned()))
    } else {
        check_name(&file_ref.name)
    }
}

fn stored_name(record: &FileRecord, target: &StoredRef) -> String {
    if !record.name.trim().is_empty() {
        return record.name.clone();
    }
    match target {
        StoredRef::Blob(hash) => {
            let short: String = hash.as_str().chars().take(BLOB_NAME_CHARS).collect();
            format!("{UNNAMED_FILE}-{short}")
        },
        StoredRef::Upload(_) | StoredRef::Document { .. } => UNNAMED_FILE.to_owned(),
    }
}

fn pinned_ref(file_ref: &FileRef, pinned: &StoredRef) -> FileRef {
    match pinned {
        StoredRef::Document {
            revision: Some(revision),
            ..
        } => FileRef {
            revision_id: revision.as_str().to_owned(),
            ..file_ref.clone()
        },
        StoredRef::Upload(_) | StoredRef::Blob(_) | StoredRef::Document { .. } => file_ref.clone(),
    }
}
