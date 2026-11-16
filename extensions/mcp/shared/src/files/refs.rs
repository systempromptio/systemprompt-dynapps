//! [`FileRef`]: how a tool names a file, in its input and in its output.
//!
//! It is one flat object of strings, empty meaning unset, and is always
//! inlined into the schema that embeds it, so a tool taking files stays
//! declarable to every provider (no `$ref`, no type lists). Exactly one
//! source is set: `upload_id` (from the server's upload tool), `blob_ref`
//! (from `documents_upload`), `document_id` with an optional `revision_id`
//! (one of the caller's documents), or `content_base64` for a body under the
//! inline limit. `name` and `mime` name the file where it lands.

use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use systemprompt::identifiers::{ContentHash, DocumentId, DocumentRevisionId, DocumentUploadId};

use super::error::FilesError;

const SOURCES: &str = "upload_id, blob_ref, document_id or content_base64";

/// A file a tool takes or hands back, by reference (inline only when tiny).
/// Its `Debug` shows the length of `content_base64`, never the content.
#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(inline)]
pub struct FileRef {
    #[schemars(
        description = "Upload id returned by the server's upload tool, once the client has sent the file and completed the upload."
    )]
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub upload_id: String,
    #[schemars(description = "blob_ref returned by documents_upload.")]
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub blob_ref: String,
    #[schemars(description = "Id of one of your own documents.")]
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub document_id: String,
    #[schemars(description = "Revision of document_id. Empty means its current revision.")]
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub revision_id: String,
    #[schemars(
        description = "The file itself as base64, only for small files (64 KB by default). Anything larger goes through the upload tool."
    )]
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub content_base64: String,
    #[schemars(description = "File name to use. Empty keeps the stored name.")]
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    #[schemars(description = "MIME type to use. Empty keeps the stored type.")]
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub mime: String,
}

/// Where a [`FileRef`] points once its fields are checked.
#[derive(Clone, PartialEq, Eq)]
pub enum FileSource {
    Stored(StoredRef),
    Inline(String),
}

impl fmt::Debug for FileRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileRef")
            .field("upload_id", &self.upload_id)
            .field("blob_ref", &self.blob_ref)
            .field("document_id", &self.document_id)
            .field("revision_id", &self.revision_id)
            .field("content_base64_len", &self.content_base64.len())
            .field("mime", &self.mime)
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for FileSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stored(target) => f.debug_tuple("Stored").field(target).finish(),
            Self::Inline(encoded) => f.debug_tuple("Inline").field(&encoded.len()).finish(),
        }
    }
}

/// A file held by the document store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoredRef {
    Upload(DocumentUploadId),
    Blob(ContentHash),
    Document {
        id: DocumentId,
        revision: Option<DocumentRevisionId>,
    },
}

impl StoredRef {
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::Upload(id) => format!("upload:{id}"),
            Self::Blob(hash) => format!("blob:{hash}"),
            Self::Document { id, .. } => format!("document:{id}"),
        }
    }
}

impl FileRef {
    #[must_use]
    pub fn upload(id: &DocumentUploadId) -> Self {
        Self {
            upload_id: id.as_str().to_owned(),
            ..Self::default()
        }
    }

    #[must_use]
    pub fn document(id: &DocumentId, revision: &DocumentRevisionId) -> Self {
        Self {
            document_id: id.as_str().to_owned(),
            revision_id: revision.as_str().to_owned(),
            ..Self::default()
        }
    }

    pub fn source(&self) -> Result<FileSource, FilesError> {
        let set = [
            &self.upload_id,
            &self.blob_ref,
            &self.document_id,
            &self.content_base64,
        ]
        .into_iter()
        .filter(|field| !field.trim().is_empty())
        .count();
        if set == 0 {
            return Err(FilesError::InvalidRef(format!(
                "a file reference names one of {SOURCES}"
            )));
        }
        if set > 1 {
            return Err(FilesError::InvalidRef(format!(
                "a file reference names exactly one of {SOURCES}, not several"
            )));
        }
        if !self.revision_id.trim().is_empty() && self.document_id.trim().is_empty() {
            return Err(FilesError::InvalidRef(
                "revision_id is only valid with document_id".to_owned(),
            ));
        }
        self.parsed()
    }

    fn parsed(&self) -> Result<FileSource, FilesError> {
        let invalid = |field: &str| FilesError::InvalidRef(format!("{field} is not a valid id"));
        if !self.upload_id.trim().is_empty() {
            return DocumentUploadId::try_new(self.upload_id.trim())
                .map(|id| FileSource::Stored(StoredRef::Upload(id)))
                .map_err(|_invalid| invalid("upload_id"));
        }
        if !self.blob_ref.trim().is_empty() {
            return ContentHash::try_new(self.blob_ref.trim().to_ascii_lowercase())
                .map(|hash| FileSource::Stored(StoredRef::Blob(hash)))
                .map_err(|_invalid| invalid("blob_ref"));
        }
        if !self.document_id.trim().is_empty() {
            let id = DocumentId::try_new(self.document_id.trim())
                .map_err(|_invalid| invalid("document_id"))?;
            let revision = match self.revision_id.trim() {
                "" => None,
                given => Some(
                    DocumentRevisionId::try_new(given).map_err(|_invalid| invalid("revision_id"))?,
                ),
            };
            return Ok(FileSource::Stored(StoredRef::Document { id, revision }));
        }
        Ok(FileSource::Inline(self.content_base64.clone()))
    }
}
