//! Public report request and response contracts.

use chrono::Utc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
// JSON: upstream report envelopes carry heterogeneous table cells.
use serde_json::Value;
use std::collections::BTreeMap;
use systemprompt::mcp::McpOutputSchema;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "report", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReportInput {
    Costs {
        #[serde(default = "seven_days")]
        days: u16,
    },
    Projects {
        cloud_id: String,
        project_key: String,
        #[serde(default = "seven_days")]
        days: u16,
        #[serde(default)]
        space_id: Option<String>,
    },
    // Why: the daily brief reads the same Jira facts as `Projects`, one traffic
    // light per project, with the AI-usage strip appended.
    Brief {
        cloud_id: String,
        project_keys: Vec<String>,
        #[serde(default = "seven_days")]
        days: u16,
    },
}

const fn seven_days() -> u16 {
    7
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CatalogInput {
    #[serde(default)]
    pub cloud_id: Option<String>,
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
    // JSON: upstream CLI and Atlassian table cells are heterogeneous protocol values.
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
    fn artifact_type() -> &'static str {
        "dashboard"
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
