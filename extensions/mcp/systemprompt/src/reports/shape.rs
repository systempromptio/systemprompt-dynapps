//! Public report request and response contracts.

use chrono::Utc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
// JSON: variable-shape report table cells
use serde_json::Value;
use std::collections::BTreeMap;
use systemprompt::mcp::McpOutputSchema;

// Why: Claude Code drops a tool whose inputSchema is a top-level `oneOf` with
// no `properties`, which a serde-tagged enum emits; keep this a flat struct.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReportInput {
    #[schemars(
        description = "Which report to run. `costs` is the only kind: AI usage, spend, models and sessions from this platform's own audit tables."
    )]
    pub report: ReportKind,
    // Why: schemars renders `Option<u16>` as a type list, which Gemini/Vertex
    // rejects in a function declaration, refusing the whole server's tools.
    #[schemars(description = "Lookback window in days, 1-90. Defaults to 7.")]
    #[serde(default = "seven_days")]
    pub days: u16,
}

// Why: schemars `inline` keeps the enum out of `$defs`; Gemini rejects a `$ref`
// into `$defs` as an unknown keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum ReportKind {
    Costs,
}

const fn seven_days() -> u16 {
    7
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SourceStatus {
    pub source: String,
    pub complete: bool,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ReportTable {
    pub title: String,
    pub columns: Vec<String>,
    // JSON: variable-shape CLI and Atlassian table cells
    pub rows: Vec<BTreeMap<String, Value>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ReportOutput {
    pub report: String,
    pub title: String,
    pub checked_at: String,
    pub period: String,
    pub complete: bool,
    pub sources: Vec<SourceStatus>,
    pub metrics: BTreeMap<String, u64>,
    #[serde(default)]
    pub highlights: BTreeMap<String, String>,
    pub tables: Vec<ReportTable>,
    // Why: serde `default` lets a stored artifact without this field still
    // deserialise; a failure in the renderer's read-back costs the dashboard.
    #[serde(default)]
    pub request: Option<ReportInput>,
}

impl ReportOutput {
    pub(crate) fn new(report: &str, title: &str, period: String) -> Self {
        Self {
            report: report.into(),
            title: title.into(),
            checked_at: Utc::now().to_rfc3339(),
            period,
            complete: true,
            sources: Vec::new(),
            metrics: BTreeMap::new(),
            highlights: BTreeMap::new(),
            tables: Vec::new(),
            request: None,
        }
    }

    pub(crate) fn source(&mut self, source: &str, complete: bool, warning: Option<String>) {
        self.complete &= complete;
        self.sources.push(SourceStatus {
            source: source.into(),
            complete,
            warning,
        });
    }
}

impl McpOutputSchema for ReportOutput {
    // Why: core's ingest wraps a built-in `dashboard` type in a `tool_result`
    // envelope that fails this tool's advertised schema; a report is its own shape.
    fn artifact_type() -> &'static str {
        "report"
    }
    fn artifact_title(&self) -> Option<String> {
        Some(self.title.clone())
    }
    fn text_body(&self) -> Option<String> {
        serde_json::to_string(self).ok()
    }
}

pub(crate) fn table(title: &str, values: Vec<Value>) -> ReportTable {
    let rows: Vec<BTreeMap<String, Value>> = values
        .into_iter()
        .map(|v| match v {
            Value::Object(map) => map.into_iter().collect(),
            other => BTreeMap::from([("value".into(), other)]),
        })
        .collect();
    let columns = rows
        .iter()
        .flat_map(|r| r.keys().cloned())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    ReportTable {
        title: title.into(),
        columns,
        rows,
    }
}
