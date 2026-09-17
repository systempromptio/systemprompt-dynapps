//! One artifact with its body, findings and the rows of the same call.

use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use systemprompt::identifiers::{ArtifactId, ContextId, McpExecutionId, SessionId, UserId};

use super::ArtifactRow;

#[derive(Debug, Clone, FromRow)]
pub struct ArtifactFindingRow {
    pub phase: String,
    pub severity: String,
    pub category: String,
    pub scanner: String,
    pub path: Option<String>,
    pub excerpt: Option<String>,
    pub redacted: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ArtifactDetail {
    pub row: ArtifactRow,
    pub trace_id: Option<String>,
    pub last_seen_source: Option<String>,
    pub payload_sha256: Option<String>,
    // JSON: the stored artifact body, typed per `artifact_type`.
    pub body: Option<serde_json::Value>,
    pub execution_status: Option<String>,
    pub execution_time_ms: Option<i32>,
    pub error_message: Option<String>,
    pub intended_at: Option<DateTime<Utc>>,
    pub governance_decision_id: Option<String>,
    pub governance_decision: Option<String>,
    pub findings: Vec<ArtifactFindingRow>,
}

// Why: the detail query's own row, flattened by `FromRow`; it is folded into
// `ArtifactDetail` so the page reads the same `ArtifactRow` the list does.
#[derive(Debug, FromRow)]
struct DetailRow {
    artifact_id: ArtifactId,
    mcp_execution_id: McpExecutionId,
    ai_tool_call_id: Option<String>,
    request_id: Option<String>,
    user_id: Option<UserId>,
    user_label: Option<String>,
    session_id: Option<SessionId>,
    context_id: Option<ContextId>,
    trace_id: Option<String>,
    tool_name: Option<String>,
    server_name: String,
    artifact_type: String,
    title: Option<String>,
    source: String,
    last_seen_source: Option<String>,
    correlation: Option<String>,
    client_kind: Option<String>,
    is_structured: bool,
    has_ui_resource: bool,
    is_error: bool,
    payload_bytes: Option<i32>,
    payload_sha256: Option<String>,
    secret_redactions: i32,
    // JSON: the stored artifact body.
    body: Option<serde_json::Value>,
    execution_status: Option<String>,
    execution_time_ms: Option<i32>,
    error_message: Option<String>,
    intended_at: Option<DateTime<Utc>>,
    governance_decision_id: Option<String>,
    governance_decision: Option<String>,
    skill: Option<String>,
    created_at: DateTime<Utc>,
}

pub async fn find_artifact(
    pool: &PgPool,
    artifact_id: &ArtifactId,
) -> Result<Option<ArtifactDetail>, sqlx::Error> {
    let Some(r) = sqlx::query_as::<_, DetailRow>(include_str!("detail.sql"))
        .bind(artifact_id.as_str())
        .fetch_optional(pool)
        .await?
    else {
        return Ok(None);
    };
    let findings = list_artifact_findings(pool, artifact_id).await?;
    Ok(Some(fold(r, findings)))
}

async fn list_artifact_findings(
    pool: &PgPool,
    artifact_id: &ArtifactId,
) -> Result<Vec<ArtifactFindingRow>, sqlx::Error> {
    sqlx::query_as::<_, ArtifactFindingRow>(
        "SELECT phase, severity, category, scanner, path, excerpt, redacted, created_at \
         FROM mcp_artifact_findings WHERE artifact_id = $1 ORDER BY created_at",
    )
    .bind(artifact_id.as_str())
    .fetch_all(pool)
    .await
}

fn fold(r: DetailRow, findings: Vec<ArtifactFindingRow>) -> ArtifactDetail {
    ArtifactDetail {
        row: ArtifactRow {
            artifact_id: r.artifact_id,
            mcp_execution_id: r.mcp_execution_id,
            ai_tool_call_id: r.ai_tool_call_id,
            request_id: r.request_id,
            user_id: r.user_id,
            user_label: r.user_label,
            session_id: r.session_id,
            context_id: r.context_id,
            tool_name: r.tool_name,
            server_name: r.server_name,
            artifact_type: r.artifact_type,
            title: r.title,
            source: r.source,
            correlation: r.correlation,
            client_kind: r.client_kind,
            is_structured: r.is_structured,
            has_ui_resource: r.has_ui_resource,
            is_error: r.is_error,
            payload_bytes: r.payload_bytes,
            secret_redactions: r.secret_redactions,
            finding_count: i64::try_from(findings.len()).unwrap_or(i64::MAX),
            skill: r.skill,
            created_at: r.created_at,
            total_rows: 1,
        },
        trace_id: r.trace_id,
        last_seen_source: r.last_seen_source,
        payload_sha256: r.payload_sha256,
        body: r.body,
        execution_status: r.execution_status,
        execution_time_ms: r.execution_time_ms,
        error_message: r.error_message,
        intended_at: r.intended_at,
        governance_decision_id: r.governance_decision_id,
        governance_decision: r.governance_decision,
        findings,
    }
}
