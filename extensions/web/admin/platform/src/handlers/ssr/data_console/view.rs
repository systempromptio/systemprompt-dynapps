//! Typed view models shared by Platform Logs and SQL.
use crate::repositories::data_console::browse::Selection;
use crate::repositories::data_console::catalog::Table;
use crate::repositories::data_console::clear::ClearPreview;
use crate::repositories::data_console::clear_plan::ClearPlan;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(super) struct Cell {
    pub value: String,
    pub limited: bool,
    pub href: Option<String>,
}
#[derive(Debug, Serialize)]
pub(super) struct Row {
    pub cells: Vec<Cell>,
    pub detail: Option<String>,
}
#[derive(Debug, Serialize)]
pub(super) struct DetailField {
    pub name: String,
    pub value: String,
    pub limited: bool,
}
#[derive(Debug, Serialize)]
pub(super) struct DataPage {
    pub page: &'static str,
    pub title: &'static str,
    pub is_logs: bool,
    pub tables: Vec<TableOption>,
    pub table: String,
    pub selection: String,
    pub from: String,
    pub until: String,
    pub time_column: String,
    pub filter_column: String,
    pub filter_value: String,
    pub level: String,
    pub module: String,
    pub context_filter: String,
    pub session_filter: String,
    pub trace_id: String,
    pub columns: Vec<String>,
    pub rows: Vec<Row>,
    pub next_url: Option<String>,
    pub refresh_url: String,
    pub no_primary: bool,
    pub limited: bool,
    pub notice: Option<String>,
    pub query: String,
    pub query_rows: Vec<String>,
    pub detail: Vec<DetailField>,
    pub preview: Option<ClearPreview>,
    pub cleared: Option<ClearPlan>,
}
#[derive(Debug, Serialize)]
pub(super) struct TableOption {
    pub label: String,
    pub selected: bool,
}
impl DataPage {
    pub(super) fn new(logs: bool) -> Self {
        Self {
            page: if logs { "logs" } else { "sql" },
            title: if logs { "Logs" } else { "SQL" },
            is_logs: logs,
            tables: Vec::new(),
            table: String::new(),
            selection: String::new(),
            from: String::new(),
            until: String::new(),
            time_column: String::new(),
            filter_column: String::new(),
            filter_value: String::new(),
            level: String::new(),
            module: String::new(),
            context_filter: String::new(),
            session_filter: String::new(),
            trace_id: String::new(),
            columns: Vec::new(),
            rows: Vec::new(),
            next_url: None,
            refresh_url: if logs { "/admin/logs" } else { "/admin/sql" }.into(),
            no_primary: false,
            limited: false,
            notice: None,
            query: String::new(),
            query_rows: Vec::new(),
            detail: Vec::new(),
            preview: None,
            cleared: None,
        }
    }
    pub(super) fn selection(&mut self, selection: &Selection) -> Result<(), serde_json::Error> {
        self.table.clone_from(&selection.table);
        self.selection = serde_json::to_string(selection)?;
        Ok(())
    }
    pub(super) fn tables(&mut self, tables: Vec<Table>) {
        self.tables = tables
            .into_iter()
            .map(|t| TableOption {
                selected: t.label() == self.table,
                label: t.label(),
            })
            .collect();
    }
}
