//! `/admin/analytics?tab=tools` — the per-tool and per-server tables, both
//! read over the same scope and window as the rest of the dashboard.

use async_trait::async_trait;

use super::{live_dataset, plan};
use crate::repositories::analytics::site::tools::{list_tool_servers, list_tool_stats};
use systemprompt_web_admin_core::error::AdminResult;
use systemprompt_web_admin_core::export::model::{
    Cell, CellKind, Column, DataSet, ExportContext, Table, Window,
};
use systemprompt_web_admin_core::export::registry::DataSetRegistration;

pub(crate) struct ToolStats;
pub(crate) struct ToolServers;

inventory::submit! {
    DataSetRegistration {
        order: 16,
        dataset: &ToolStats,
    }
}
inventory::submit! {
    DataSetRegistration {
        order: 17,
        dataset: &ToolServers,
    }
}

const TOOL_COLUMNS: &[Column] = &[
    Column::new("server", "Server", CellKind::Text),
    Column::new("tool", "Tool", CellKind::Text),
    Column::new("executions", "Executions", CellKind::Integer),
    Column::new("succeeded", "Succeeded", CellKind::Integer),
    Column::new("failed", "Failed", CellKind::Integer),
    Column::new("pending", "Pending", CellKind::Integer).optional(),
    Column::new("p50_ms", "p50 (ms)", CellKind::Decimal),
    Column::new("p95_ms", "p95 (ms)", CellKind::Decimal),
    Column::new("distinct_users", "Users", CellKind::Integer),
];

live_dataset!(
    ToolStats,
    "analytics-tools",
    "Tools",
    "One row per server and tool: executions, outcomes, latency and people over the window.",
    TOOL_COLUMNS,
    |ctx| {
        let p = plan(ctx).await?;
        let (rows, total) = list_tool_stats(ctx.pool, p.range, &p.scope, ctx.limit, 0).await?;
        Ok(Table {
            rows: rows
                .iter()
                .map(|r| {
                    vec![
                        r.server_name.as_str().into(),
                        r.tool_name.as_str().into(),
                        r.executions.into(),
                        r.succeeded.into(),
                        r.failed.into(),
                        r.pending.into(),
                        Cell::opt_decimal(r.p50_ms),
                        Cell::opt_decimal(r.p95_ms),
                        r.distinct_users.into(),
                    ]
                })
                .collect(),
            total,
        })
    }
);

const SERVER_COLUMNS: &[Column] = &[
    Column::new("server", "Server", CellKind::Text),
    Column::new("executions", "Executions", CellKind::Integer),
    Column::new("succeeded", "Succeeded", CellKind::Integer),
    Column::new("tools", "Tools", CellKind::Integer),
    Column::new("distinct_users", "Users", CellKind::Integer),
];

live_dataset!(
    ToolServers,
    "analytics-tool-servers",
    "Tool servers",
    "One row per tool server: executions, successes, tools used and people over the window.",
    SERVER_COLUMNS,
    |ctx| {
        let p = plan(ctx).await?;
        let rows = list_tool_servers(ctx.pool, p.range, &p.scope).await?;
        Ok(Table::complete(
            rows.iter()
                .map(|r| {
                    vec![
                        r.server_name.as_str().into(),
                        r.executions.into(),
                        r.succeeded.into(),
                        r.tools.into(),
                        r.distinct_users.into(),
                    ]
                })
                .collect(),
        ))
    }
);
