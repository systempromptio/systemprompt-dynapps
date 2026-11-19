//! The pieces [`DocumentsFileStore`] is assembled from: reading one
//! document's record as the request's actor, the upload session's record,
//! the revision author a kept file is attributed to, and the translation of
//! core's upload, document and link errors into [`FilesError`].
//!
//! A document the caller may not open, trashed or capped away from the
//! agent, reads as not found, the same as one that does not exist.
//! Infrastructure failures are logged here with their cause and answered
//! as [`FilesError::Unavailable`].

use systemprompt::files::documents::DocumentsError;
use systemprompt::files::documents::models::{RevisionAttribution, RevisionOrigin};
use systemprompt::files::documents::service::Author;
use systemprompt::files::uploads::{UploadError, UploadSession, UploadStatus};
use systemprompt::identifiers::{DocumentId, DocumentRevisionId, McpExecutionId};
use systemprompt::models::documents::ActorKind;
use systemprompt::models::execution::context::RequestContext;
use systemprompt::runtime::documents::actor::act_as_request;
use systemprompt::runtime::documents::links::LinkIssueError;
use systemprompt::runtime::documents::refusal::{RefusalCode, refusal_of};

use super::documents_store::DocumentsFileStore;
use super::error::FilesError;
use super::ops::caller;
use super::refs::StoredRef;
use super::store::{FileRecord, FileState, UploadSlot};

impl DocumentsFileStore {
    pub(super) async fn describe_document(
        &self,
        ctx: &RequestContext,
        id: &DocumentId,
        revision: Option<&DocumentRevisionId>,
    ) -> Result<FileRecord, FilesError> {
        let who = caller(ctx)?;
        let service = &self.documents;
        let (view, pinned) = act_as_request(ctx, async {
            let view = service.get(who, id, ctx.trace_id()).await?;
            let pinned = match revision {
                Some(asked) => Some(service.revision(who, id, asked, ctx.trace_id()).await?),
                None => view.head.clone(),
            };
            Ok::<_, DocumentsError>((view, pinned))
        })
        .await
        .map_err(|error| documents_error(&error))?;
        let pinned = pinned.ok_or_else(|| FilesError::NotFound {
            what: format!("a revision of document {id}"),
        })?;
        let size = service
            .revision_size(&pinned)
            .await
            .map_err(|error| documents_error(&error))?
            .and_then(|size| u64::try_from(size).ok());
        let state = if view.document.trashed_at.is_some() {
            FileState::Gone
        } else {
            FileState::Ready { since: None }
        };
        Ok(FileRecord {
            owner: view.document.owner_user_id,
            name: view.document.name,
            mime: view.document.mime_type,
            size_bytes: size,
            state,
            pinned: StoredRef::Document {
                id: id.clone(),
                revision: Some(pinned.id),
            },
        })
    }
}

pub(super) fn author(ctx: &RequestContext, exec_id: &McpExecutionId) -> Author {
    let present = |value: &str| !value.is_empty();
    Author {
        actor_kind: ActorKind::Agent,
        origin: RevisionOrigin::McpExecution(exec_id.clone()),
        attribution: RevisionAttribution {
            session_id: Some(ctx.session_id().clone()).filter(|id| present(id.as_str())),
            context_id: Some(ctx.context_id().clone()).filter(|id| present(id.as_str())),
            trace_id: Some(ctx.trace_id().clone()).filter(|id| present(id.as_str())),
            task_id: ctx.task_id().cloned(),
            on_behalf_of: Some(ctx.user_id().clone()).filter(|id| present(id.as_str())),
        },
    }
}

pub(super) fn upload_slot(session: &UploadSession) -> UploadSlot {
    UploadSlot {
        upload_id: session.id.clone(),
        owner: session.owner_user_id.clone(),
        part_size: session.part_size,
        part_count: session.part_count(),
        expires_at: session.expires_at,
        open: session.status == UploadStatus::Open,
    }
}

pub(super) fn upload_record(session: &UploadSession) -> FileRecord {
    let state = match session.status {
        UploadStatus::Completed => FileState::Ready {
            since: Some(session.finished_at.unwrap_or(session.created_at)),
        },
        UploadStatus::Open | UploadStatus::Completing => FileState::Pending {
            expires_at: session.expires_at,
        },
        UploadStatus::Refused | UploadStatus::Aborted | UploadStatus::Expired => FileState::Expired,
    };
    FileRecord {
        owner: session.owner_user_id.clone(),
        name: session.file_name.clone(),
        mime: session
            .detected_mime
            .clone()
            .unwrap_or_else(|| session.declared_mime.clone()),
        size_bytes: Some(session.declared_size),
        state,
        pinned: StoredRef::Upload(session.id.clone()),
    }
}

pub(super) fn upload_error(error: &UploadError) -> FilesError {
    match error {
        UploadError::NotFound | UploadError::BlobNotOwned(_) => FilesError::NotFound {
            what: "the file".to_owned(),
        },
        UploadError::TooLarge { size, limit } => FilesError::TooLarge {
            size: *size,
            limit: *limit,
        },
        UploadError::Storage(_)
        | UploadError::Repository(_)
        | UploadError::Blob(_)
        | UploadError::Interrupted(_)
        | UploadError::Unanswered(_) => {
            tracing::error!(error = %error, cause = ?std::error::Error::source(error), "the upload store failed");
            FilesError::Unavailable
        },
        other => FilesError::Refused {
            code: other.code().to_owned(),
            detail: other.to_string(),
        },
    }
}

pub(super) fn documents_error(error: &DocumentsError) -> FilesError {
    if let DocumentsError::NameTaken { name } = error {
        return FilesError::NameTaken { name: name.clone() };
    }
    let Some(refused) = refusal_of(error) else {
        tracing::error!(error = %error, cause = ?std::error::Error::source(error), "the document store failed");
        return FilesError::Unavailable;
    };
    match refused.refusal.code {
        RefusalCode::NotFound
        | RefusalCode::Trashed
        | RefusalCode::Forbidden
        | RefusalCode::AgentForbidden => FilesError::NotFound {
            what: "the document".to_owned(),
        },
        code => FilesError::Refused {
            code: code.as_str().to_owned(),
            detail: refused.message,
        },
    }
}

pub(super) fn link_error(error: &LinkIssueError) -> FilesError {
    tracing::error!(
        error = %error,
        cause = ?std::error::Error::source(error),
        "the API did not sign a file link; the MCP server must host the documents toolset"
    );
    FilesError::LinkUnavailable
}
