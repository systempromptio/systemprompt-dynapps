//! `/admin/artifacts` — one row per tool result in the window.

use async_trait::async_trait;

use super::requests::time_range;
use crate::error::AdminResult;
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::handlers::ssr::ssr_artifacts::export_rows;
use crate::repositories::analytics::artifacts::ArtifactRow;
use systemprompt::identifiers::{ContextId, SessionId, UserId};

pub(crate) struct Artifacts;

const COLUMNS: &[Column] = &[
    Column::new("created_at", "Time", CellKind::Timestamp),
    Column::new("artifact_id", "Artifact", CellKind::Text),
    Column::new("execution_id", "Execution", CellKind::Text).optional(),
    Column::new("request_id", "Request", CellKind::Text).optional(),
    Column::new("session_id", "Session", CellKind::Text),
    Column::new("context_id", "Context", CellKind::Text).optional(),
    Column::new("user_id", "User", CellKind::Text),
    Column::new("user_label", "Name", CellKind::Text).optional(),
    Column::new("server", "Server", CellKind::Text),
    Column::new("tool", "Tool", CellKind::Text),
    Column::new("skill", "Skill", CellKind::Text),
    Column::new("type", "Type", CellKind::Text),
    Column::new("title", "Title", CellKind::Text),
    Column::new("source", "Source", CellKind::Text),
    Column::new("client", "Client", CellKind::Text).optional(),
    Column::new("structured", "Structured", CellKind::Bool),
    Column::new("ui", "UI resource", CellKind::Bool).optional(),
    Column::new("error", "Error", CellKind::Bool),
    Column::new("bytes", "Payload bytes", CellKind::Integer),
    Column::new("redactions", "Secret redactions", CellKind::Integer).optional(),
    Column::new("findings", "Findings", CellKind::Integer),
];

fn row(r: &ArtifactRow) -> Vec<Cell> {
    vec![
        r.created_at.into(),
        r.artifact_id.as_str().into(),
        r.mcp_execution_id.as_str().into(),
        Cell::opt_text(r.request_id.as_deref()),
        Cell::opt_text(r.session_id.as_ref().map(SessionId::as_str)),
        Cell::opt_text(r.context_id.as_ref().map(ContextId::as_str)),
        Cell::opt_text(r.user_id.as_ref().map(UserId::as_str)),
        Cell::opt_text(r.user_label.as_deref()),
        r.server_name.as_str().into(),
        Cell::opt_text(r.tool_name.as_deref()),
        Cell::opt_text(r.skill.as_deref()),
        r.artifact_type.as_str().into(),
        Cell::opt_text(r.title.as_deref()),
        r.source.as_str().into(),
        Cell::opt_text(r.client_kind.as_deref()),
        r.is_structured.into(),
        r.has_ui_resource.into(),
        r.is_error.into(),
        Cell::opt_int(r.payload_bytes),
        r.secret_redactions.into(),
        r.finding_count.into(),
    ]
}

#[async_trait]
impl DataSet for Artifacts {
    fn id(&self) -> &'static str {
        "artifacts"
    }
    fn title(&self) -> &'static str {
        "Artifacts"
    }
    fn columns(&self) -> &'static [Column] {
        COLUMNS
    }
    fn window(&self) -> Window {
        Window::Live
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let (rows, total) = export_rows(
            ctx.pool,
            ctx.user,
            ctx.query()?,
            time_range(ctx)?,
            ctx.limit,
        )
        .await?;
        Ok(Table {
            rows: rows.iter().map(row).collect(),
            total,
        })
    }
}
