//! The file layer every MCP server in this repo shares, so file bytes never
//! travel through the model's context.
//!
//! A tool takes a file as a [`FileRef`](crate::files::FileRef) and hands one
//! back as a [`FileOut`](crate::files::FileOut).
//! [`McpFiles`](crate::files::McpFiles) sits on core's document store runtime:
//! `request_upload` opens a resumable upload owned by the caller and answers
//! the signed links a client sends the bytes to; `resolve_ref` turns a
//! [`FileRef`](crate::files::FileRef) (an upload, a blob, one of the caller's
//! documents, or base64 up to the inline limit) into a byte stream;
//! `store_download` keeps bytes a server fetched as a document of the caller
//! and answers a signed download link. Every operation, refusals included,
//! writes one audit row.
//!
//! The decisions (ownership, expiry, size, MIME, inline limit, audit) are
//! taken here, once, for every server; the store behind them is the
//! [`FileStore`](crate::files::FileStore) port, implemented over core by
//! [`DocumentsFileStore`](crate::files::DocumentsFileStore).
//! Nothing here knows which upstream a server talks to.

mod audit;
mod documents_store;
mod documents_support;
mod error;
mod ops;
mod outputs;
mod policy;
mod refs;
mod resolve;
mod store;

use std::sync::Arc;

use systemprompt::identifiers::{JwtToken, McpExecutionId, McpServerId};
use systemprompt::models::execution::context::RequestContext;
use systemprompt::system::HostContext;

pub use audit::{DbFileAudit, FileAction, FileAuditEvent, FileAuditSink};
pub use documents_store::DocumentsFileStore;
pub use error::FilesError;
pub use outputs::{
    DownloadedFile, FileOut, ResolvedFile, ResolvedParts, UploadRequest, UploadTicket,
};
pub use policy::{
    DEFAULT_INLINE_MAX_BYTES, DEFAULT_MAX_FILE_BYTES, DEFAULT_REF_TTL_HOURS, FilePolicy,
    OCTET_STREAM, check_name, download_name, normalize_mime, numbered_name, tagged_name,
};
pub use refs::{FileRef, FileSource, StoredRef};
pub use store::{
    FileRecord, FileState, FileStore, NewStoredFile, SignedDownload, StoredFile, UploadSlot,
    byte_stream,
};

/// The file layer of one MCP server.
#[derive(Debug, Clone)]
pub struct McpFiles {
    store: Arc<dyn FileStore>,
    audit: Arc<dyn FileAuditSink>,
    server: McpServerId,
    policy: FilePolicy,
}

/// The collaborators an [`McpFiles`] is built from.
#[derive(Debug, Clone)]
pub struct McpFilesParts {
    pub store: Arc<dyn FileStore>,
    pub audit: Arc<dyn FileAuditSink>,
    pub server: McpServerId,
    pub policy: FilePolicy,
}

/// The tool call a file operation serves: the authenticated request, the
/// tool's name and execution, and the hop token the proxy minted for the
/// call, which the API needs to sign a link.
#[derive(Debug, Clone, Copy)]
pub struct FileCall<'a> {
    pub ctx: &'a RequestContext,
    pub tool: &'a str,
    pub exec_id: &'a McpExecutionId,
    pub hop: Option<&'a JwtToken>,
}

impl McpFiles {
    #[must_use]
    pub fn new(parts: McpFilesParts) -> Self {
        Self {
            store: parts.store,
            audit: parts.audit,
            server: parts.server,
            policy: parts.policy,
        }
    }

    pub fn from_host(
        host: &HostContext,
        server: McpServerId,
        policy: FilePolicy,
    ) -> Result<Self, FilesError> {
        let store = DocumentsFileStore::from_host(host, &server)?;
        Ok(Self::new(McpFilesParts {
            store: Arc::new(store),
            audit: Arc::new(DbFileAudit::new(Arc::clone(host.db_pool()))),
            server,
            policy,
        }))
    }

    #[must_use]
    pub const fn policy(&self) -> &FilePolicy {
        &self.policy
    }

    pub fn decode_inline(&self, encoded: &str) -> Result<Vec<u8>, FilesError> {
        self.policy.decode_inline(encoded)
    }
}
