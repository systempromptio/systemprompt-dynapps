//! Reads for the Artifacts pages: every tool result the platform ingested,
//! as one row of the tool-call ledger joined to its typed artifact.
//!
//! The ledger (`tool_call_ledger`, core) joins intent, execution and result
//! on the client `tool_use_id`; these queries read the artifact side of it
//! with the owner's display name, and the detail read adds the stored body,
//! the scanner findings and the governance decision of the same call. The
//! SQL lives beside this file so each query is read whole.

mod detail;
mod list;

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use systemprompt::identifiers::{ArtifactId, ContextId, McpExecutionId, SessionId, UserId};

pub use detail::{ArtifactDetail, ArtifactFindingRow, find_artifact};
pub use list::{get_artifact_stats, list_artifacts};

/// Which artifacts a list read covers. Everything is optional except the
/// window; `structured_only` is the page's default lens.
#[derive(Debug, Clone, Default)]
pub struct ArtifactFilter {
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub user_id: Option<UserId>,
    pub subject_ids: Option<Vec<String>>,
    pub session_id: Option<SessionId>,
    pub tool_name: Option<String>,
    pub source: Option<String>,
    pub skill: Option<String>,
    pub structured_only: bool,
    pub errors_only: bool,
}

#[derive(Debug, Clone, FromRow)]
pub struct ArtifactRow {
    pub artifact_id: ArtifactId,
    pub mcp_execution_id: McpExecutionId,
    pub ai_tool_call_id: Option<String>,
    pub request_id: Option<String>,
    pub user_id: Option<UserId>,
    pub user_label: Option<String>,
    pub session_id: Option<SessionId>,
    pub context_id: Option<ContextId>,
    pub tool_name: Option<String>,
    pub server_name: String,
    pub artifact_type: String,
    pub title: Option<String>,
    pub source: String,
    pub correlation: Option<String>,
    pub client_kind: Option<String>,
    pub is_structured: bool,
    pub has_ui_resource: bool,
    pub is_error: bool,
    pub payload_bytes: Option<i32>,
    pub secret_redactions: i32,
    pub finding_count: i64,
    pub skill: Option<String>,
    pub created_at: DateTime<Utc>,
    pub total_rows: i64,
}

#[derive(Debug, Clone, Copy, Default, FromRow)]
pub struct ArtifactStats {
    pub artifacts: i64,
    pub structured: i64,
    pub errors: i64,
    pub ui: i64,
    pub inferred: i64,
    pub with_findings: i64,
    pub bytes: i64,
    pub tools: i64,
    pub sessions: i64,
}
