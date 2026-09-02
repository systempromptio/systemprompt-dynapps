//! `add_knowledge_document`: one hand-authored document, persisted into its
//! category's tree and indexed immediately (FR-ING-6).
//!
//! The bytes are written under `<category>/uploads/<file_name>` before
//! anything is indexed, so the document is a file like any other: the next
//! directory sync converges on the same `doc_id`, a rebuild re-reads it, and
//! a prune of stale users cannot touch it. Indexing runs the same pipeline
//! the sync uses, then stamps the index so a reader sees the new version at
//! once — no rebuild, no restart.

use super::grants::GrantPolicy;
use super::pipeline::{DocumentJob, process_document};
use super::sources::files::{DocumentSpec, document_from_text};
use super::{Ingestor, writer};
use crate::config::SourceKind;
use crate::error::KnowledgeBankError;
use crate::ingest::parsers::{SUPPORTED_SUFFIXES, parse_named, suffix_of};
use crate::principal::{Principal, Subject};
use crate::tools::AddKnowledgeDocumentInput;
use base64::Engine;
use std::path::{Path, PathBuf};

pub const MAX_UPLOAD_BYTES: usize = 20 * 1024 * 1024;
const UPLOAD_DIR: &str = "uploads";

/// One document to add, already validated by its
/// `TryFrom<AddKnowledgeDocumentInput>`.
#[derive(Debug, Clone)]
pub struct UploadRequest {
    pub source_type: String,
    pub file_name: String,
    pub bytes: Vec<u8>,
    pub title: Option<String>,
    pub overwrite: bool,
}

/// What the tool reports back.
#[derive(Debug, Clone)]
pub struct UploadReport {
    pub doc_id: String,
    pub uri: String,
    pub title: String,
    pub chunks: usize,
    pub replaced: bool,
    pub granted_to: Vec<Subject>,
    pub version: String,
}

impl TryFrom<AddKnowledgeDocumentInput> for UploadRequest {
    type Error = KnowledgeBankError;

    // Why: this is the validation of the raw tool arguments. Exactly one of
    // `content` (text) and `content_base64` (binary) must be given; the file
    // name is a bare name with a supported suffix; the decoded size is capped.
    fn try_from(input: AddKnowledgeDocumentInput) -> Result<Self, KnowledgeBankError> {
        validate_file_name(&input.file_name)?;
        let bytes = match (input.content, input.content_base64) {
            (Some(text), None) => text.into_bytes(),
            (None, Some(encoded)) => base64::engine::general_purpose::STANDARD
                .decode(encoded.trim())
                // Why: DecodeError's variants describe byte offsets no caller
                // can act on; "not base64" is the whole message.
                // lint-ok: error-adapt
                .map_err(|e| KnowledgeBankError::InvalidInput(format!("content_base64: {e}")))?,
            (Some(_), Some(_)) => {
                return Err(KnowledgeBankError::InvalidInput(
                    "pass either content or content_base64, not both".to_owned(),
                ));
            },
            (None, None) => {
                return Err(KnowledgeBankError::InvalidInput(
                    "pass content (text) or content_base64 (pdf/docx)".to_owned(),
                ));
            },
        };
        if bytes.is_empty() {
            return Err(KnowledgeBankError::InvalidInput(
                "the document is empty".to_owned(),
            ));
        }
        if bytes.len() > MAX_UPLOAD_BYTES {
            return Err(KnowledgeBankError::InvalidInput(format!(
                "the document is {} bytes; the limit is {MAX_UPLOAD_BYTES}",
                bytes.len()
            )));
        }
        Ok(Self {
            source_type: input.source_type,
            file_name: input.file_name,
            bytes,
            title: input.title.filter(|t| !t.trim().is_empty()),
            overwrite: input.overwrite,
        })
    }
}

pub fn validate_file_name(file_name: &str) -> Result<(), KnowledgeBankError> {
    let valid_chars = !file_name.is_empty()
        && file_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    if !valid_chars || file_name.starts_with('.') || file_name.contains("..") {
        return Err(KnowledgeBankError::InvalidInput(format!(
            "file_name '{file_name}' must be a bare name of letters, digits, '.', '_' and '-'"
        )));
    }
    let suffix = suffix_of(file_name);
    if !SUPPORTED_SUFFIXES.contains(&suffix.as_str()) {
        return Err(KnowledgeBankError::InvalidInput(format!(
            "file_name '{file_name}' must end in one of {}",
            SUPPORTED_SUFFIXES.join(", ")
        )));
    }
    Ok(())
}

struct Placement<'a> {
    source: &'a crate::config::SourceConfig,
    root: PathBuf,
    rel: String,
    target: PathBuf,
    replaced: bool,
}

impl Ingestor {
    fn place_upload(&self, request: &UploadRequest) -> Result<Placement<'_>, KnowledgeBankError> {
        let source = self
            .config
            .sources
            .get(&request.source_type)
            .ok_or_else(|| KnowledgeBankError::UnknownSource(request.source_type.clone()))?;
        if source.kind != SourceKind::Files {
            return Err(KnowledgeBankError::InvalidInput(format!(
                "source '{}' is synced from {:?}; documents can only be added to files categories",
                request.source_type, source.kind
            )));
        }
        let rel = format!("{UPLOAD_DIR}/{}", request.file_name);
        let root = source.resolved_path(&request.source_type, &self.config.files);
        let target = root.join(&rel);
        let replaced = target.exists();
        if replaced && !request.overwrite {
            return Err(KnowledgeBankError::InvalidInput(format!(
                "'{rel}' already exists in '{}'; pass overwrite to replace it",
                request.source_type
            )));
        }
        Ok(Placement {
            source,
            root,
            rel,
            target,
            replaced,
        })
    }

    pub async fn add_document(
        &self,
        caller: &Principal,
        request: UploadRequest,
    ) -> Result<UploadReport, KnowledgeBankError> {
        let Placement {
            source,
            root,
            rel,
            target,
            replaced,
        } = self.place_upload(&request)?;
        let body = {
            let name = request.file_name.clone();
            let bytes = request.bytes.clone();
            tokio::task::spawn_blocking(move || parse_named(&name, &bytes))
                .await
                // Why: JoinError is variant-less; a panicking parser is this
                // one document's failure.
                // lint-ok: error-adapt
                .map_err(|e| KnowledgeBankError::Internal(format!("parser task: {e}")))?
                .map_err(|e| KnowledgeBankError::InvalidInput(e.to_string()))?
        };
        write_atomically(&target, &request.bytes).await?;

        let mtime = super::sources::files::mtime(&target).await;
        let mut document = document_from_text(&DocumentSpec {
            source_type: &request.source_type,
            root: &root,
            rel: &rel,
            strategy: source.strategy,
            body,
            mtime,
        })
        .ok_or_else(|| {
            KnowledgeBankError::InvalidInput("the document contains no readable text".to_owned())
        })?;
        if let Some(title) = &request.title {
            document.title.clone_from(title);
        }

        let job = DocumentJob {
            pool: &self.pool,
            model: self.model.as_ref(),
            config: &self.config,
            source,
            document: &document,
        };
        let chunks = process_document(&job).await?;
        let subjects = source.configured_subjects();
        writer::apply_grants(
            &self.pool,
            &document.doc_id,
            &GrantPolicy::Configured(subjects.clone()),
        )
        .await?;
        let version = writer::stamp_index_state(&self.pool).await?;
        tracing::info!(
            user_id = %caller.user_id,
            doc_id = %document.doc_id,
            chunks,
            replaced,
            "knowledge document added"
        );
        Ok(UploadReport {
            doc_id: document.doc_id,
            uri: document.uri,
            title: document.title,
            chunks,
            replaced,
            granted_to: subjects,
            version,
        })
    }
}

async fn write_atomically(target: &Path, bytes: &[u8]) -> Result<(), KnowledgeBankError> {
    let parent = target
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    tokio::fs::create_dir_all(&parent)
        .await
        // Why: io::Error carries no variant an operator acts on beyond the
        // path, which this message adds.
        // lint-ok: error-adapt
        .map_err(|e| KnowledgeBankError::Internal(format!("create {}: {e}", parent.display())))?;
    let tmp = target.with_extension("tmp-upload");
    tokio::fs::write(&tmp, bytes)
        .await
        // Why: same adaptation as above.
        // lint-ok: error-adapt
        .map_err(|e| KnowledgeBankError::Internal(format!("write {}: {e}", tmp.display())))?;
    tokio::fs::rename(&tmp, target)
        .await
        // Why: same adaptation as above.
        // lint-ok: error-adapt
        .map_err(|e| KnowledgeBankError::Internal(format!("rename {}: {e}", target.display())))
}
