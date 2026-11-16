//! [`FilesError`]: why a file operation was refused or failed, worded for the
//! model that has to act on it, and the MCP error each one is answered with.
//!
//! A reference the caller may not open reads as not found, never as
//! forbidden, so a refusal does not confirm that another user's file
//! exists. Infrastructure failures carry no cause here: the store logs it
//! and answers [`FilesError::Unavailable`].

use rmcp::ErrorData;
use systemprompt::identifiers::DocumentUploadId;

use crate::reply;

/// A refused or failed file operation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FilesError {
    #[error("{0}")]
    InvalidRef(String),
    #[error("{0}")]
    InvalidName(String),
    #[error("`{given}` is not a MIME type; give it as type/subtype, e.g. application/pdf")]
    InvalidMime { given: String },
    #[error("files of type {mime} are not accepted here; accepted types: {accepted}")]
    MimeNotAllowed { mime: String, accepted: String },
    #[error("the file is empty")]
    Empty,
    #[error("the file is {size} bytes; the limit is {limit} bytes")]
    TooLarge { size: u64, limit: u64 },
    #[error(
        "content_base64 carries {size} bytes, over the {limit}-byte inline limit: call \
         {upload_tool} for an upload link, send the file there, and pass the upload_id it \
         returns instead"
    )]
    InlineTooLarge {
        size: u64,
        limit: u64,
        upload_tool: String,
    },
    #[error("content_base64 is not valid base64")]
    InvalidBase64,
    #[error("sha256 must be the file's SHA-256 digest as 64 hexadecimal characters")]
    InvalidChecksum,
    #[error("{what} was not found, or is not yours")]
    NotFound { what: String },
    #[error(
        "upload {upload} is not complete yet: if every part is not yet sent to parts_url and \
         complete_url not yet posted, do that; if completion is under way, retry shortly"
    )]
    NotReady { upload: DocumentUploadId },
    #[error("{what} is no longer available (expired, aborted or refused); call {upload_tool} for a new upload")]
    Expired { what: String, upload_tool: String },
    #[error("a file named {name} already exists")]
    NameTaken { name: String },
    #[error("the document store refused the file ({code}): {detail}")]
    Refused { code: String, detail: String },
    #[error("this call carries no signed-in user; files are read and kept as the caller")]
    Anonymous,
    #[error(
        "the document store is disabled: set files.documents.enabled: true in \
         services/config/files.yaml"
    )]
    StoreDisabled,
    #[error("the signed link could not be issued; try the call again")]
    LinkUnavailable,
    #[error("the file store failed; try the call again")]
    Unavailable,
}

impl FilesError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidRef(_) => "invalid_ref",
            Self::InvalidName(_) => "invalid_name",
            Self::InvalidMime { .. } => "invalid_mime",
            Self::MimeNotAllowed { .. } => "mime_not_allowed",
            Self::Empty => "empty",
            Self::TooLarge { .. } => "too_large",
            Self::InlineTooLarge { .. } => "inline_too_large",
            Self::InvalidBase64 => "invalid_base64",
            Self::InvalidChecksum => "invalid_checksum",
            Self::NotFound { .. } => "not_found",
            Self::NotReady { .. } => "not_ready",
            Self::Expired { .. } => "expired",
            Self::NameTaken { .. } => "name_taken",
            Self::Refused { .. } => "refused",
            Self::Anonymous => "anonymous",
            Self::StoreDisabled => "store_disabled",
            Self::LinkUnavailable => "link_unavailable",
            Self::Unavailable => "unavailable",
        }
    }
}

impl From<FilesError> for ErrorData {
    fn from(error: FilesError) -> Self {
        let message = error.to_string();
        match error {
            FilesError::NotFound { .. } => reply::not_found(message),
            FilesError::Anonymous => reply::forbidden(message),
            FilesError::StoreDisabled | FilesError::LinkUnavailable | FilesError::Unavailable => {
                reply::failure(message)
            },
            _ => reply::invalid(message),
        }
    }
}
