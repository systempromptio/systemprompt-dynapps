//! The audit trail of the file layer: one `user_activity` row per file
//! operation, refusals included, in the `mcp_access` category beside the
//! server's own access rows.
//!
//! A row is attributed to the signed-in caller; a refusal of a call that
//! carried no user is attributed to the anonymous principal, like
//! `record_mcp_access_rejected`, never to whatever user id the request
//! context holds. A row names the file by its reference (`upload:<id>`, `blob:<hash>`,
//! `document:<id>`, `inline`), never by its name, and a refusal by its
//! error code, so no file name or address reaches the audit table. Actions
//! are drawn from the set the activity timeline decodes (`created`, `used`,
//! `imported`, `rejected`); a new word would make that read fail.

use std::fmt;

use async_trait::async_trait;
use systemprompt::database::DbPool;
use systemprompt::identifiers::{McpServerId, UserId};

use crate::{AuditMetadata, resolve_anonymous_principal};
use crate::repositories::{McpAccessParams, insert_mcp_access};

const FILE_ENTITY: &str = "file";

/// What happened to a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileAction {
    UploadOpened,
    Read,
    Stored,
    Refused,
}

impl FileAction {
    #[must_use]
    pub const fn activity(self) -> &'static str {
        match self {
            Self::UploadOpened => "created",
            Self::Read => "used",
            Self::Stored => "imported",
            Self::Refused => "rejected",
        }
    }

    const fn verb(self) -> &'static str {
        match self {
            Self::UploadOpened => "Opened an upload of",
            Self::Read => "Read",
            Self::Stored => "Stored",
            Self::Refused => "Refused",
        }
    }
}

/// One audited file operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileAuditEvent {
    pub user_id: Option<UserId>,
    pub server: McpServerId,
    pub tool: String,
    pub action: FileAction,
    pub file: String,
    pub size_bytes: Option<u64>,
    pub reason: Option<String>,
}

impl FileAuditEvent {
    #[must_use]
    pub fn description(&self) -> String {
        let size = self
            .size_bytes
            .map_or_else(String::new, |size| format!(" ({size} bytes)"));
        format!(
            "{} file {}{size} on {} for '{}'",
            self.action.verb(),
            self.file,
            self.server,
            self.tool
        )
    }
}

/// Where file audit events are written.
#[async_trait]
pub trait FileAuditSink: Send + Sync + fmt::Debug {
    async fn record(&self, event: &FileAuditEvent);
}

/// [`FileAuditSink`] over `user_activity`. Best-effort, like
/// `record_mcp_access`: a failed insert is logged and the operation goes on.
#[derive(Debug, Clone)]
pub struct DbFileAudit {
    pool: DbPool,
}

impl DbFileAudit {
    #[must_use]
    pub const fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FileAuditSink for DbFileAudit {
    async fn record(&self, event: &FileAuditEvent) {
        let metadata = AuditMetadata {
            tool_name: event.tool.clone(),
            server: event.server.to_string(),
            reason: event.reason.clone(),
        };
        let pool = self.pool.pool();
        let server = event.server.to_string();
        let user_id = match &event.user_id {
            Some(user_id) => user_id.clone(),
            None => {
                let Some(anonymous) =
                    resolve_anonymous_principal(pool.as_ref(), &server, &event.tool).await
                else {
                    return;
                };
                anonymous
            },
        };
        let description = event.description();
        let params = McpAccessParams {
            user_id: &user_id,
            action: event.action.activity(),
            entity_type: FILE_ENTITY,
            entity_name: &event.file,
            description: &description,
            metadata: &metadata,
        };
        if let Err(e) = insert_mcp_access(pool.as_ref(), &params).await {
            tracing::warn!(error = %e, "Failed to record MCP file access event (non-fatal)");
        }
    }
}
