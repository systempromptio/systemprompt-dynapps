//! `/admin/mcp` per-tool split — one row per server and tool over the window.
//! `?server=` narrows it to one server, and it always loads the whole
//! aggregate up to its cap, ignoring the preview limit.

use async_trait::async_trait;

use super::mcp::ServerQuery;
use crate::repositories::mcp::runtime::{McpToolStat, list_mcp_tool_stats};
use systemprompt_web_admin_core::error::AdminResult;
use systemprompt_web_admin_core::export::model::{
    Cell, CellKind, Column, DataSet, ExportContext, Table, Window,
};
use systemprompt_web_admin_core::export::registry::DataSetRegistration;
use systemprompt_web_admin_core::export::window::time_range;

pub(crate) struct McpTools;

inventory::submit! {
    DataSetRegistration {
        order: 20,
        dataset: &McpTools,
    }
}

const TOOL_COLUMNS: &[Column] = &[
    Column::new("server", "Server", CellKind::Text),
    Column::new("tool", "Tool", CellKind::Text),
    Column::new("calls", "Calls", CellKind::Integer),
    Column::new("failed", "Failed or timed out", CellKind::Integer),
    Column::new("attested", "Attested", CellKind::Integer),
    Column::new("users", "Users", CellKind::Integer),
    Column::new("avg_ms", "Avg (ms)", CellKind::Decimal),
    Column::new("p95_ms", "p95 (ms)", CellKind::Decimal),
    Column::new("max_ms", "Max (ms)", CellKind::Integer),
    Column::new("payload_bytes", "Payload bytes", CellKind::Integer),
    Column::new("last_call_at", "Last call", CellKind::Timestamp),
];

fn tool_row(r: &McpToolStat) -> Vec<Cell> {
    vec![
        r.server_name.as_str().into(),
        r.tool_name.as_str().into(),
        r.calls.into(),
        r.failures.into(),
        r.attested.into(),
        r.distinct_users.into(),
        Cell::opt_decimal(r.avg_ms),
        Cell::opt_decimal(r.p95_ms),
        Cell::opt_int(r.max_ms),
        r.payload_bytes.into(),
        Cell::opt_time(r.last_call_at),
    ]
}

#[async_trait]
impl DataSet for McpTools {
    fn id(&self) -> &'static str {
        "mcp-tools"
    }
    fn title(&self) -> &'static str {
        "MCP tools"
    }
    fn description(&self) -> &'static str {
        "One row per server and tool: calls, errors, attestation, latency and payload over the window."
    }
    fn columns(&self) -> &'static [Column] {
        TOOL_COLUMNS
    }
    fn window(&self) -> Window {
        Window::Live
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let query: ServerQuery = ctx.query()?;
        let range = time_range(ctx)?;
        let row_cap = self.cap();
        let rows = list_mcp_tool_stats(
            ctx.pool,
            query.server.as_deref().filter(|s| !s.is_empty()),
            range.from,
            range.to,
            row_cap,
        )
        .await?;
        Ok(Table::complete(rows.iter().map(tool_row).collect()))
    }
}
