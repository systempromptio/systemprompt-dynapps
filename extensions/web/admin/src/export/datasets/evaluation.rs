//! `/admin/analysis/evaluation/runs` — every judge run.

use async_trait::async_trait;
use serde::Deserialize;

use crate::error::AdminResult;
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::repositories::evaluation::runs::{EvaluationRunRow, RunsFilter, list_runs_paged};

pub(crate) struct Runs;

#[derive(Debug, Default, Deserialize)]
struct Filters {
    criterion: Option<String>,
    status: Option<String>,
    host: Option<String>,
    session: Option<String>,
}

fn clean(value: Option<String>) -> Option<String> {
    value.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
}

const COLUMNS: &[Column] = &[
    Column::new("created_at", "Time", CellKind::Timestamp),
    Column::new("id", "Run", CellKind::Text),
    Column::new("session_id", "Session", CellKind::Text),
    Column::new("user_id", "User", CellKind::Text),
    Column::new("host", "Host", CellKind::Text),
    Column::new("criterion", "Criterion", CellKind::Text),
    Column::new("attempt", "Attempt", CellKind::Integer).optional(),
    Column::new("status", "Status", CellKind::Text),
    Column::new("score", "Score", CellKind::Integer),
    Column::new("score_source", "Score source", CellKind::Text).optional(),
    Column::new("provider", "Provider", CellKind::Text),
    Column::new("model", "Model", CellKind::Text),
    Column::new("input_tokens", "Input tokens", CellKind::Integer),
    Column::new("output_tokens", "Output tokens", CellKind::Integer),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
    Column::new("latency_ms", "Latency (ms)", CellKind::Integer),
    Column::new("trigger", "Trigger", CellKind::Text),
    Column::new("requested_by", "Requested by", CellKind::Text).optional(),
    Column::new("error", "Error", CellKind::Text),
];

fn row(r: &EvaluationRunRow) -> Vec<Cell> {
    vec![
        r.created_at.into(),
        r.id.as_str().into(),
        r.session_id.as_str().into(),
        r.user_id.as_str().into(),
        r.host.as_str().into(),
        r.criterion.as_str().into(),
        r.attempt.into(),
        r.status.as_str().into(),
        Cell::opt_int(r.score),
        r.score_source.as_str().into(),
        r.provider.as_str().into(),
        r.model.as_str().into(),
        Cell::opt_int(r.input_tokens),
        Cell::opt_int(r.output_tokens),
        r.cost_microdollars.map_or(Cell::Empty, Cell::Money),
        Cell::opt_int(r.latency_ms),
        r.trigger.as_str().into(),
        Cell::opt_text(r.requested_by.as_ref().map(|u| u.as_str())),
        Cell::opt_text(r.error.as_deref()),
    ]
}

#[async_trait]
impl DataSet for Runs {
    fn id(&self) -> &'static str {
        "evaluation-runs"
    }
    fn title(&self) -> &'static str {
        "Judge runs"
    }
    fn columns(&self) -> &'static [Column] {
        COLUMNS
    }
    fn window(&self) -> Window {
        Window::None
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let f: Filters = ctx.query()?;
        let filter = RunsFilter {
            criterion: clean(f.criterion),
            status: clean(f.status),
            host: clean(f.host),
            session: clean(f.session),
            limit: ctx.limit,
            offset: 0,
        };
        let (rows, total) = list_runs_paged(ctx.pool, &filter).await?;
        Ok(Table {
            rows: rows.iter().map(row).collect(),
            total,
        })
    }
}
