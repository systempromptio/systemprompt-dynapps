//! What a server fetched from its upstream, on its way to the caller.
//!
//! `store_download` keeps it as a document of the caller and hands back a
//! reference; `return_inline` audits bytes the server returns to the model
//! itself (text small enough to read). Both write the same `imported` row,
//! and both refuse a call with no signed-in user; a server whose
//! `return_inline` is refused must not return the bytes.

use systemprompt::identifiers::DocumentUploadId;

use super::audit::FileAction;
use super::error::FilesError;
use super::ops::{Audit, caller};
use super::outputs::{DownloadedFile, FileOut};
use super::policy::{download_name, numbered_name, tagged_name};
use super::refs::FileRef;
use super::resolve::INLINE_LABEL;
use super::store::{NewStoredFile, SignedDownload, StoredFile};
use super::{FileCall, McpFiles};

const NUMBERED_NAME_ATTEMPTS: u32 = 4;
const UNIQUE_TAG_CHARS: usize = 8;
const DOWNLOAD_LABEL: &str = "download";

fn unique_tag() -> String {
    DocumentUploadId::generate()
        .as_str()
        .chars()
        .take(UNIQUE_TAG_CHARS)
        .collect()
}

impl McpFiles {
    pub async fn store_download(
        &self,
        call: FileCall<'_>,
        file: DownloadedFile,
    ) -> Result<FileOut, FilesError> {
        let size = file.bytes.len() as u64;
        let outcome = self.keep(call, file).await;
        let label = outcome.as_ref().map_or_else(
            |_refused| DOWNLOAD_LABEL.to_owned(),
            |out| format!("document:{}", out.file_ref.document_id),
        );
        let audit = Audit {
            action: FileAction::Stored,
            file: label,
            size_bytes: Some(size),
        };
        self.audited(call, audit, outcome).await
    }

    pub async fn return_inline(
        &self,
        call: FileCall<'_>,
        size_bytes: u64,
    ) -> Result<(), FilesError> {
        let outcome = caller(call.ctx).map(|_signed_in| ());
        let audit = Audit {
            action: FileAction::Returned,
            file: INLINE_LABEL.to_owned(),
            size_bytes: Some(size_bytes),
        };
        self.audited(call, audit, outcome).await
    }

    async fn keep(&self, call: FileCall<'_>, file: DownloadedFile) -> Result<FileOut, FilesError> {
        caller(call.ctx)?;
        self.policy.check_size(file.bytes.len() as u64)?;
        let base = download_name(&file.name);
        let mut new_file = NewStoredFile {
            name: base.clone(),
            mime: self.policy.download_mime(&file.mime)?,
            bytes: file.bytes,
            exec_id: call.exec_id.clone(),
        };
        let stored = self.store_unique(call, &mut new_file, &base).await?;
        let link = self.link(call, &stored).await;
        Ok(FileOut {
            file_ref: FileRef::document(&stored.document_id, &stored.revision_id),
            name: stored.name,
            mime: stored.mime,
            size_bytes: stored.size_bytes,
            download_url: link.as_ref().map(|signed| signed.url.clone()),
            expires_at: link.map(|signed| signed.expires_at),
        })
    }

    async fn store_unique(
        &self,
        call: FileCall<'_>,
        file: &mut NewStoredFile,
        base: &str,
    ) -> Result<StoredFile, FilesError> {
        let mut attempt = 1;
        loop {
            match self.store.store(call.ctx, file).await {
                Err(FilesError::NameTaken { .. }) if attempt <= NUMBERED_NAME_ATTEMPTS => {
                    attempt += 1;
                    file.name = if attempt <= NUMBERED_NAME_ATTEMPTS {
                        numbered_name(base, attempt)
                    } else {
                        tagged_name(base, &unique_tag())
                    };
                },
                outcome => return outcome,
            }
        }
    }

    async fn link(&self, call: FileCall<'_>, stored: &StoredFile) -> Option<SignedDownload> {
        match self
            .store
            .sign_download(call.hop, &stored.document_id, &stored.revision_id)
            .await
        {
            Ok(link) => Some(link),
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    document_id = %stored.document_id,
                    "download link not issued; the stored document is still referenced"
                );
                None
            },
        }
    }
}
