//! [`DocumentsFileStore`]: [`FileStore`] over core's document store.
//!
//! Uploads and blobs go through the upload service, documents through the
//! document service acting as the request's actor (so an agent's cap on a
//! document holds here too), and signed links through the API's link
//! issuer, so every check core makes on its own `documents_*` tools applies
//! to a file this layer reads or keeps. A kept file is a `file`-format
//! document in the caller's root folder.
//!
//! The API signs a link only for a server whose deployment hosts the
//! `documents` toolset, so a server using this store declares it in its
//! `services/mcp/` file.
//!
//! A blob is described as ready with no date, so `ref_ttl` never applies to a
//! `blob_ref`: core's upload service answers who owns a blob and its type
//! (`BlobOrigin`), not when the upload that made it finished. A `blob_ref`
//! lasts as long as core keeps the blob.

use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use systemprompt::files::documents::DocumentService;
use systemprompt::files::documents::service::{FileCreateRequest, StreamRequest};
use systemprompt::files::uploads::{NewUpload, UploadService};
use systemprompt::identifiers::{
    DocumentId, DocumentRevisionId, DocumentUploadId, JwtToken, McpServerId,
};
use systemprompt::models::documents::DocumentFormat;
use systemprompt::models::execution::context::RequestContext;
use systemprompt::runtime::documents::actor::act_as_request;
use systemprompt::runtime::documents::download::Rendition;
use systemprompt::runtime::documents::links::{
    LinkIssuer, LinkTarget, issue_download_link, issue_upload_links,
};
use systemprompt::runtime::documents::upload_links::SignedUploadLinks;
use systemprompt::system::HostContext;
use systemprompt::traits::FileByteStream;

use super::documents_support::{
    author, documents_error, link_error, upload_error, upload_record, upload_slot,
};
use super::error::FilesError;
use super::ops::caller;
use super::refs::StoredRef;
use super::store::{
    FileRecord, FileState, FileStore, NewStoredFile, SignedDownload, StoredFile, UploadSlot,
    byte_stream,
};

/// [`FileStore`] over core's upload and document services.
#[derive(Clone)]
pub struct DocumentsFileStore {
    pub(super) documents: Arc<DocumentService>,
    uploads: Arc<UploadService>,
    links: Arc<dyn LinkIssuer>,
}

impl fmt::Debug for DocumentsFileStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DocumentsFileStore")
            .field("links", &self.links)
            .finish_non_exhaustive()
    }
}

impl DocumentsFileStore {
    #[must_use]
    pub const fn new(
        documents: Arc<DocumentService>,
        uploads: Arc<UploadService>,
        links: Arc<dyn LinkIssuer>,
    ) -> Self {
        Self {
            documents,
            uploads,
            links,
        }
    }

    pub fn from_host(host: &HostContext, server: &McpServerId) -> Result<Self, FilesError> {
        if !host.files_config().documents_config().is_enabled() {
            return Err(FilesError::StoreDisabled);
        }
        let links = host.link_issuer(server).map_err(|error| {
            tracing::error!(error = %error, server = %server, "the API link client could not be built");
            FilesError::Unavailable
        })?;
        let runtime = host.documents();
        Ok(Self::new(
            Arc::clone(runtime.service()),
            Arc::clone(runtime.uploads()),
            links,
        ))
    }
}

#[async_trait]
impl FileStore for DocumentsFileStore {
    async fn open_upload(
        &self,
        ctx: &RequestContext,
        upload: NewUpload,
    ) -> Result<UploadSlot, FilesError> {
        let who = caller(ctx)?;
        let session = self
            .uploads
            .create(&who.id, upload, ctx.trace_id())
            .await
            .map_err(|error| upload_error(&error))?;
        Ok(upload_slot(&session))
    }

    async fn find_upload(
        &self,
        ctx: &RequestContext,
        upload: &DocumentUploadId,
    ) -> Result<UploadSlot, FilesError> {
        let who = caller(ctx)?;
        let progress = self
            .uploads
            .progress(&who.id, upload)
            .await
            .map_err(|error| upload_error(&error))?;
        Ok(upload_slot(&progress.session))
    }

    async fn sign_upload(
        &self,
        hop: Option<&JwtToken>,
        upload: &DocumentUploadId,
    ) -> Result<SignedUploadLinks, FilesError> {
        issue_upload_links(self.links.as_ref(), hop, upload)
            .await
            .map_err(|error| link_error(&error))
    }

    async fn describe(
        &self,
        ctx: &RequestContext,
        target: &StoredRef,
    ) -> Result<FileRecord, FilesError> {
        let who = caller(ctx)?;
        match target {
            StoredRef::Upload(id) => {
                let progress = self
                    .uploads
                    .progress(&who.id, id)
                    .await
                    .map_err(|error| upload_error(&error))?;
                Ok(upload_record(&progress.session))
            },
            StoredRef::Blob(hash) => {
                let origin = self
                    .uploads
                    .require_owned_blob(&who.id, hash)
                    .await
                    .map_err(|error| upload_error(&error))?;
                Ok(FileRecord {
                    owner: who.id.clone(),
                    name: String::new(),
                    mime: origin.mime,
                    size_bytes: None,
                    state: FileState::Ready { since: None },
                    pinned: target.clone(),
                })
            },
            StoredRef::Document { id, revision } => {
                self.describe_document(ctx, id, revision.as_ref()).await
            },
        }
    }

    async fn open(
        &self,
        ctx: &RequestContext,
        target: &StoredRef,
        max_bytes: u64,
    ) -> Result<FileByteStream, FilesError> {
        let who = caller(ctx)?;
        match target {
            StoredRef::Upload(id) => {
                let (_, read) = self
                    .uploads
                    .open_blob(&who.id, id, None)
                    .await
                    .map_err(|error| upload_error(&error))?;
                Ok(read.body)
            },
            StoredRef::Blob(hash) => {
                let (bytes, _) = self
                    .uploads
                    .read_owned_blob(&who.id, hash, DocumentFormat::File, max_bytes)
                    .await
                    .map_err(|error| upload_error(&error))?;
                Ok(byte_stream(bytes))
            },
            StoredRef::Document { id, revision } => {
                let request = StreamRequest {
                    document: id,
                    revision: revision.as_ref(),
                    range: None,
                };
                let (_, read) = act_as_request(
                    ctx,
                    self.documents.open_stream(who, request, ctx.trace_id()),
                )
                .await
                .map_err(|error| documents_error(&error))?;
                Ok(read.body)
            },
        }
    }

    async fn store(
        &self,
        ctx: &RequestContext,
        file: &NewStoredFile,
    ) -> Result<StoredFile, FilesError> {
        let who = caller(ctx)?;
        let request = FileCreateRequest {
            name: file.name.clone(),
            folder: None,
            format: Some(DocumentFormat::File),
            mime: file.mime.clone(),
            bytes: file.bytes.clone(),
            author: author(ctx, &file.exec_id),
            message: None,
        };
        let (document, revision) = act_as_request(
            ctx,
            Box::pin(
                self.documents
                    .create_from_file(who, request, ctx.trace_id()),
            ),
        )
        .await
        .map_err(|error| documents_error(&error))?;
        Ok(StoredFile {
            document_id: document.id,
            revision_id: revision.id,
            name: document.name,
            mime: document.mime_type,
            size_bytes: file.bytes.len() as u64,
        })
    }

    async fn sign_download(
        &self,
        hop: Option<&JwtToken>,
        document: &DocumentId,
        revision: &DocumentRevisionId,
    ) -> Result<SignedDownload, FilesError> {
        let target = LinkTarget::Download {
            document_id: document.clone(),
            revision_id: revision.clone(),
            rendition: Rendition::Native,
        };
        let (url, expires_at) = issue_download_link(self.links.as_ref(), hop, target)
            .await
            .map_err(|error| link_error(&error))?;
        Ok(SignedDownload { url, expires_at })
    }
}
