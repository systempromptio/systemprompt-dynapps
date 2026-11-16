//! A file store and an audit sink held in memory, so the file layer's
//! decisions (ownership, expiry, limits, audit) are tested without Postgres
//! or the API. The fake answers whatever it holds for any caller: refusing
//! another user's file is the layer's job, not the store's.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{Duration, Utc};
use systemprompt::files::uploads::NewUpload;
use systemprompt::identifiers::{
    Actor, AgentName, ContextId, DocumentId, DocumentRevisionId, DocumentUploadId, JwtToken,
    McpExecutionId, McpServerId, SessionId, TraceId, UserId,
};
use systemprompt::models::auth::AuthenticatedUser;
use systemprompt::models::execution::context::RequestContext;
use systemprompt::runtime::documents::upload_links::SignedUploadLinks;
use systemprompt::traits::FileByteStream;
use systemprompt_mcp_shared::files::{
    FileAuditEvent, FileAuditSink, FilePolicy, FileRecord, FileState, FileStore, FilesError,
    McpFiles, McpFilesParts, NewStoredFile, SignedDownload, StoredFile, StoredRef, UploadSlot,
    byte_stream,
};

pub(crate) const ALICE: &str = "00000000-0000-4000-8000-00000000a11c";
pub(crate) const BOB: &str = "00000000-0000-4000-8000-000000000b0b";
pub(crate) const UPLOAD_TOOL: &str = "test_request_upload";
pub(crate) const PART_SIZE: u64 = 8 * 1024 * 1024;

#[derive(Debug, Default)]
pub(crate) struct FakeState {
    pub(crate) records: HashMap<String, (FileRecord, Vec<u8>)>,
    pub(crate) uploads: HashMap<String, UploadSlot>,
    pub(crate) opened: u32,
    pub(crate) stored: Vec<String>,
    pub(crate) taken_names: HashSet<String>,
    pub(crate) links_down: bool,
}

#[derive(Debug, Default)]
pub(crate) struct FakeStore {
    pub(crate) state: Mutex<FakeState>,
}

impl FakeStore {
    pub(crate) fn with<T>(&self, f: impl FnOnce(&mut FakeState) -> T) -> T {
        f(&mut self.state.lock().expect("fake store lock"))
    }

    pub(crate) fn put(&self, record: FileRecord, bytes: &[u8]) {
        self.with(|state| {
            state
                .records
                .insert(record.pinned.label(), (record, bytes.to_vec()));
        });
    }

    pub(crate) fn complete(&self, upload: &DocumentUploadId, bytes: &[u8], finished_ago: Duration) {
        let label = StoredRef::Upload(upload.clone()).label();
        self.with(|state| {
            let (record, stored) = state.records.get_mut(&label).expect("open upload");
            record.state = FileState::Ready {
                since: Some(Utc::now() - finished_ago),
            };
            record.size_bytes = Some(bytes.len() as u64);
            *stored = bytes.to_vec();
            if let Some(slot) = state.uploads.get_mut(upload.as_str()) {
                slot.open = false;
            }
        });
    }
}

fn owner_of(ctx: &RequestContext) -> Result<UserId, FilesError> {
    ctx.user
        .as_ref()
        .map(|user| user.id.clone())
        .ok_or(FilesError::Anonymous)
}

#[async_trait]
impl FileStore for FakeStore {
    async fn open_upload(
        &self,
        ctx: &RequestContext,
        upload: NewUpload,
    ) -> Result<UploadSlot, FilesError> {
        let owner = owner_of(ctx)?;
        let id = DocumentUploadId::generate();
        let expires_at = Utc::now() + Duration::hours(24);
        let slot = UploadSlot {
            upload_id: id.clone(),
            owner: owner.clone(),
            part_size: PART_SIZE,
            part_count: u32::try_from(upload.size.div_ceil(PART_SIZE)).expect("parts"),
            expires_at,
            open: true,
        };
        let record = FileRecord {
            owner,
            name: upload.name,
            mime: upload.mime,
            size_bytes: Some(upload.size),
            state: FileState::Pending { expires_at },
            pinned: StoredRef::Upload(id.clone()),
        };
        self.with(|state| {
            state.uploads.insert(id.as_str().to_owned(), slot.clone());
            state
                .records
                .insert(record.pinned.label(), (record, Vec::new()));
        });
        Ok(slot)
    }

    async fn find_upload(
        &self,
        _ctx: &RequestContext,
        upload: &DocumentUploadId,
    ) -> Result<UploadSlot, FilesError> {
        self.with(|state| state.uploads.get(upload.as_str()).cloned())
            .ok_or_else(|| FilesError::NotFound {
                what: "the upload".to_owned(),
            })
    }

    async fn sign_upload(
        &self,
        _hop: Option<&JwtToken>,
        upload: &DocumentUploadId,
    ) -> Result<SignedUploadLinks, FilesError> {
        if self.with(|state| state.links_down) {
            return Err(FilesError::LinkUnavailable);
        }
        let base = format!("https://api.test/api/v1/uploads/{upload}");
        Ok(SignedUploadLinks {
            token: "signed-upload-token".to_owned(),
            status_url: base.clone(),
            parts_url: format!("{base}/parts/{{n}}"),
            complete_url: format!("{base}/complete"),
            content_url: format!("{base}/content"),
            expires_at: Utc::now() + Duration::minutes(15),
        })
    }

    async fn describe(
        &self,
        _ctx: &RequestContext,
        target: &StoredRef,
    ) -> Result<FileRecord, FilesError> {
        self.with(|state| state.records.get(&target.label()).map(|(r, _)| r.clone()))
            .ok_or_else(|| FilesError::NotFound {
                what: target.label(),
            })
    }

    async fn open(
        &self,
        _ctx: &RequestContext,
        target: &StoredRef,
        _max_bytes: u64,
    ) -> Result<FileByteStream, FilesError> {
        let bytes = self.with(|state| {
            state.opened += 1;
            state.records.get(&target.label()).map(|(_, b)| b.clone())
        });
        bytes.map(byte_stream).ok_or_else(|| FilesError::NotFound {
            what: target.label(),
        })
    }

    async fn store(
        &self,
        ctx: &RequestContext,
        file: &NewStoredFile,
    ) -> Result<StoredFile, FilesError> {
        let owner = owner_of(ctx)?;
        if self.with(|state| state.taken_names.contains(&file.name)) {
            return Err(FilesError::NameTaken {
                name: file.name.clone(),
            });
        }
        let stored = StoredFile {
            document_id: DocumentId::generate(),
            revision_id: DocumentRevisionId::generate(),
            name: file.name.clone(),
            mime: file.mime.clone(),
            size_bytes: file.bytes.len() as u64,
        };
        self.put(
            FileRecord {
                owner,
                name: stored.name.clone(),
                mime: stored.mime.clone(),
                size_bytes: Some(stored.size_bytes),
                state: FileState::Ready { since: None },
                pinned: StoredRef::Document {
                    id: stored.document_id.clone(),
                    revision: Some(stored.revision_id.clone()),
                },
            },
            &file.bytes,
        );
        self.with(|state| state.stored.push(file.name.clone()));
        Ok(stored)
    }

    async fn sign_download(
        &self,
        _hop: Option<&JwtToken>,
        document: &DocumentId,
        revision: &DocumentRevisionId,
    ) -> Result<SignedDownload, FilesError> {
        if self.with(|state| state.links_down) {
            return Err(FilesError::LinkUnavailable);
        }
        Ok(SignedDownload {
            url: format!("https://api.test/api/v1/documents/{document}/content?revision={revision}"),
            expires_at: Utc::now() + Duration::minutes(10),
        })
    }
}

#[derive(Debug, Default)]
pub(crate) struct RecordingAudit {
    pub(crate) events: Mutex<Vec<FileAuditEvent>>,
}

impl RecordingAudit {
    pub(crate) fn events(&self) -> Vec<FileAuditEvent> {
        self.events.lock().expect("audit lock").clone()
    }
}

#[async_trait]
impl FileAuditSink for RecordingAudit {
    async fn record(&self, event: &FileAuditEvent) {
        self.events.lock().expect("audit lock").push(event.clone());
    }
}

pub(crate) struct Harness {
    pub(crate) store: Arc<FakeStore>,
    pub(crate) audit: Arc<RecordingAudit>,
    pub(crate) files: McpFiles,
}

pub(crate) fn harness_with(policy: FilePolicy) -> Harness {
    let store = Arc::new(FakeStore::default());
    let audit = Arc::new(RecordingAudit::default());
    let store_port: Arc<dyn FileStore> = Arc::<FakeStore>::clone(&store);
    let audit_port: Arc<dyn FileAuditSink> = Arc::<RecordingAudit>::clone(&audit);
    let files = McpFiles::new(McpFilesParts {
        store: store_port,
        audit: audit_port,
        server: McpServerId::try_new("files-test").expect("server id"),
        policy,
    });
    Harness {
        store,
        audit,
        files,
    }
}

pub(crate) fn harness() -> Harness {
    harness_with(FilePolicy::new(UPLOAD_TOOL))
}

pub(crate) fn signed_in(user: &str) -> RequestContext {
    anonymous().with_user(AuthenticatedUser::new(
        UserId::new(user),
        "files-user".to_owned(),
        "files-user@example.test".to_owned(),
        Vec::new(),
    ))
}

pub(crate) fn anonymous() -> RequestContext {
    RequestContext::new(
        SessionId::new("files-session"),
        TraceId::new("files-trace"),
        ContextId::try_new("00000000-0000-4000-8000-0000000f11e5").expect("context id"),
        AgentName::try_new("files-agent").expect("agent name"),
        Actor::user(UserId::new("00000000-0000-4000-8000-000000000000")),
    )
}

pub(crate) fn exec_id() -> McpExecutionId {
    McpExecutionId::new("files-exec")
}

pub(crate) fn document_of(owner: &str, name: &str, mime: &str, size: u64) -> (FileRecord, DocumentId) {
    let id = DocumentId::generate();
    let record = FileRecord {
        owner: UserId::new(owner),
        name: name.to_owned(),
        mime: mime.to_owned(),
        size_bytes: Some(size),
        state: FileState::Ready { since: None },
        pinned: StoredRef::Document {
            id: id.clone(),
            revision: Some(DocumentRevisionId::generate()),
        },
    };
    (record, id)
}
