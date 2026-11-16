//! `request_upload` and `store_download`, and the audit row every operation
//! of [`McpFiles`] ends with, whether it succeeded or was refused.

use chrono::{DateTime, Utc};
use systemprompt::files::uploads::NewUpload;
use systemprompt::identifiers::{ContentHash, DocumentUploadId, UserId};
use systemprompt::models::auth::AuthenticatedUser;
use systemprompt::models::execution::context::RequestContext;
use systemprompt::runtime::documents::upload_links::UPLOAD_TOKEN_HEADER;

use super::audit::{FileAction, FileAuditEvent};
use super::error::FilesError;
use super::outputs::{DownloadedFile, FileOut, UploadRequest, UploadTicket};
use super::policy::{check_name, download_name, numbered_name, tagged_name};
use super::refs::FileRef;
use super::store::{NewStoredFile, SignedDownload, StoredFile, UploadSlot};
use super::{FileCall, McpFiles};

const NUMBERED_NAME_ATTEMPTS: u32 = 4;
const UNIQUE_TAG_CHARS: usize = 8;
const UPLOAD_LABEL: &str = "upload";
const DOWNLOAD_LABEL: &str = "download";

pub(super) fn caller(ctx: &RequestContext) -> Result<&AuthenticatedUser, FilesError> {
    ctx.user.as_ref().ok_or(FilesError::Anonymous)
}

fn unique_tag() -> String {
    DocumentUploadId::generate()
        .as_str()
        .chars()
        .take(UNIQUE_TAG_CHARS)
        .collect()
}

pub(super) fn ensure_owner(caller: &UserId, owner: &UserId, what: &str) -> Result<(), FilesError> {
    if caller == owner {
        Ok(())
    } else {
        Err(FilesError::NotFound {
            what: what.to_owned(),
        })
    }
}

impl McpFiles {
    pub async fn request_upload(
        &self,
        call: FileCall<'_>,
        request: &UploadRequest,
    ) -> Result<UploadTicket, FilesError> {
        let outcome = self.ticket(call, request).await;
        let label = outcome.as_ref().map_or_else(
            |_refused| UPLOAD_LABEL.to_owned(),
            |ticket| format!("{UPLOAD_LABEL}:{}", ticket.upload_id),
        );
        let declared = Some(request.size).filter(|size| *size > 0);
        self.audited(call, FileAction::UploadOpened, label, declared, outcome)
            .await
    }

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
        self.audited(call, FileAction::Stored, label, Some(size), outcome)
            .await
    }

    pub(super) async fn audited<T>(
        &self,
        call: FileCall<'_>,
        action: FileAction,
        file: String,
        size_bytes: Option<u64>,
        outcome: Result<T, FilesError>,
    ) -> Result<T, FilesError> {
        let (action, reason) = match &outcome {
            Ok(_) => (action, None),
            Err(error) => (FileAction::Refused, Some(error.code().to_owned())),
        };
        let user_id = call.ctx.user.as_ref().map(|user| user.id.clone());
        let event = FileAuditEvent {
            user_id,
            server: self.server.clone(),
            tool: call.tool.to_owned(),
            action,
            file,
            size_bytes,
            reason,
        };
        self.audit.record(&event).await;
        outcome
    }

    pub(super) fn expired(&self, what: String) -> FilesError {
        FilesError::Expired {
            what,
            upload_tool: self.policy.upload_tool.clone(),
        }
    }

    async fn ticket(
        &self,
        call: FileCall<'_>,
        request: &UploadRequest,
    ) -> Result<UploadTicket, FilesError> {
        let who = caller(call.ctx)?;
        let slot = if request.upload_id.trim().is_empty() {
            let upload = self.new_upload(request)?;
            self.store.open_upload(call.ctx, upload).await?
        } else {
            self.renewable(call.ctx, &who.id, request.upload_id.trim())
                .await?
        };
        let links = self.store.sign_upload(call.hop, &slot.upload_id).await?;
        let expires_at = links.expires_at.min(slot.expires_at);
        Ok(UploadTicket {
            file_ref: FileRef::upload(&slot.upload_id),
            instructions: self.instructions(&slot, expires_at),
            upload_id: slot.upload_id,
            part_size: slot.part_size,
            part_count: slot.part_count,
            parts_url: links.parts_url,
            status_url: links.status_url,
            complete_url: links.complete_url,
            upload_token: links.token,
            token_header: UPLOAD_TOKEN_HEADER.to_owned(),
            expires_at,
        })
    }

    fn new_upload(&self, request: &UploadRequest) -> Result<NewUpload, FilesError> {
        let name = check_name(&request.name)?;
        let mime = self.policy.mime(&request.mime)?;
        self.policy.check_size(request.size)?;
        let sha256 = ContentHash::try_new(request.sha256.trim().to_ascii_lowercase())
            .map_err(|_invalid| FilesError::InvalidChecksum)?;
        Ok(NewUpload {
            name,
            mime,
            size: request.size,
            sha256,
        })
    }

    async fn renewable(
        &self,
        ctx: &RequestContext,
        owner: &UserId,
        given: &str,
    ) -> Result<UploadSlot, FilesError> {
        let id = DocumentUploadId::try_new(given)
            .map_err(|_invalid| FilesError::InvalidRef("upload_id is not a valid id".to_owned()))?;
        let slot = self.store.find_upload(ctx, &id).await?;
        let label = format!("upload {id}");
        ensure_owner(owner, &slot.owner, &label)?;
        if !slot.open || slot.expires_at <= Utc::now() {
            return Err(self.expired(label));
        }
        Ok(slot)
    }

    fn instructions(&self, slot: &UploadSlot, expires_at: DateTime<Utc>) -> String {
        format!(
            "Send the file in {parts} part(s) of {size} bytes (the last one may be shorter): PUT \
             part n to parts_url with {{n}} replaced by n, then POST complete_url; every request \
             carries upload_token in the {header} header. Then pass file_ref (upload_id {id}) to \
             the tool that needs the file. The token expires at {expires}; to renew it, call \
             {tool} with this upload_id.",
            parts = slot.part_count,
            size = slot.part_size,
            header = UPLOAD_TOKEN_HEADER,
            id = slot.upload_id,
            expires = expires_at.to_rfc3339(),
            tool = self.policy.upload_tool,
        )
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
