//! The Report tab and the identity badges. The report is the observer's
//! `agent_farm_run_reports` row: whether the task completed, its summary and
//! final message, the files it wrote, its MCP calls per server split into
//! reads and writes, the governance `warn` findings and whether the summary
//! reached `DynBrain`. The run facts beside it (trigger, start and end, turns,
//! tokens, cost) come from the run rows. The badges name the client the run
//! was for, the person who launched it and the identity its credentials were
//! minted for (core's `agent_runs.user_id`).

use serde::Serialize;
// JSON: the report row's outputs and mcp_calls are JSONB the observer
// writes; each item is read field by field below.
use serde_json::Value;
use systemprompt_web_admin_core::ssr::format::{format_bytes, local_time};
use systemprompt_web_shared::format::format_cost;

use super::RunData;
use crate::handlers::ssr::farm_clients::views::{completion_badge, eval_badge};
use crate::handlers::ssr::shared::{BadgeView, trigger_badge};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IdentityView {
    pub client_name: Option<String>,
    pub client_url: Option<String>,
    pub launcher_name: String,
    pub launcher_url: String,
    pub run_as_name: String,
    pub run_as_url: String,
    pub run_as_differs: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OutputView {
    pub path: String,
    pub size: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct McpServerView {
    pub server: String,
    pub reads: i64,
    pub writes: i64,
    pub failed: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RunFactsView {
    pub trigger: BadgeView,
    pub started: String,
    pub finished: String,
    pub turns: String,
    pub tokens: String,
    pub cost: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RunReportView {
    pub has_report: bool,
    pub facts: RunFactsView,
    pub completion: BadgeView,
    pub eval: BadgeView,
    pub delivery: BadgeView,
    pub summary: String,
    pub final_message: String,
    pub recorded_local: String,
    pub outputs: Vec<OutputView>,
    pub has_outputs: bool,
    pub mcp_servers: Vec<McpServerView>,
    pub has_mcp_servers: bool,
    pub mcp_reads: i32,
    pub mcp_writes: i32,
    pub governance_warnings: i32,
}

fn text(item: &Value, key: &str) -> String {
    match item.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Null) | None => String::new(),
        Some(other) => other.to_string(),
    }
}

fn number(item: &Value, key: &str) -> i64 {
    item.get(key).and_then(Value::as_i64).unwrap_or(0)
}

pub(crate) fn delivery_badge(status: &str) -> BadgeView {
    let (tone, label) = match status {
        "delivered" => ("ok", "Delivered to DynBrain"),
        "failed" => ("err", "DynBrain delivery failed"),
        _ => ("muted", "Not delivered"),
    };
    BadgeView::new(tone, label, status)
}

fn items(value: &Value) -> &[Value] {
    value.as_array().map_or(&[], Vec::as_slice)
}

fn facts_view(data: &RunData) -> RunFactsView {
    let when =
        |t: Option<chrono::DateTime<chrono::Utc>>| t.map_or_else(|| "—".to_owned(), local_time);
    let agent_run = &data.agent_run;
    let tokens = match (agent_run.input_tokens, agent_run.output_tokens) {
        (None, None) => "—".to_owned(),
        (input, output) => format!("{} in · {} out", input.unwrap_or(0), output.unwrap_or(0)),
    };
    RunFactsView {
        trigger: trigger_badge(&data.run.triggered_via),
        started: when(agent_run.started_at.or(data.run.started_at)),
        finished: when(agent_run.finished_at.or(data.run.finished_at)),
        turns: agent_run
            .turns
            .map_or_else(|| "—".to_owned(), |t| t.to_string()),
        tokens,
        cost: format_cost(data.run.cost_microdollars),
    }
}

pub(crate) fn report_view(data: &RunData) -> RunReportView {
    let facts = facts_view(data);
    let Some(report) = &data.report else {
        return RunReportView {
            has_report: false,
            facts,
            completion: completion_badge(None),
            eval: eval_badge(None, None),
            delivery: delivery_badge(""),
            summary: String::new(),
            final_message: String::new(),
            recorded_local: String::new(),
            outputs: Vec::new(),
            has_outputs: false,
            mcp_servers: Vec::new(),
            has_mcp_servers: false,
            mcp_reads: 0,
            mcp_writes: 0,
            governance_warnings: 0,
        };
    };
    let outputs: Vec<OutputView> = items(&report.outputs)
        .iter()
        .map(|o| OutputView {
            path: text(o, "path"),
            size: o
                .get("bytes")
                .and_then(Value::as_i64)
                .map_or_else(String::new, format_bytes),
            sha256: text(o, "sha256"),
        })
        .collect();
    let mcp_servers: Vec<McpServerView> = items(&report.mcp_calls)
        .iter()
        .map(|c| McpServerView {
            server: text(c, "server"),
            reads: number(c, "reads"),
            writes: number(c, "writes"),
            failed: number(c, "failed"),
        })
        .collect();
    RunReportView {
        has_report: true,
        facts,
        completion: completion_badge(Some(report.completed)),
        eval: eval_badge(report.eval_passed, report.eval_score),
        delivery: delivery_badge(&report.delivery_status),
        summary: report.summary.clone().unwrap_or_default(),
        final_message: report.final_message.clone().unwrap_or_default(),
        recorded_local: local_time(report.created_at),
        has_outputs: !outputs.is_empty(),
        outputs,
        has_mcp_servers: !mcp_servers.is_empty(),
        mcp_servers,
        mcp_reads: report.mcp_reads,
        mcp_writes: report.mcp_writes,
        governance_warnings: report.governance_warnings,
    }
}

pub(crate) fn identity_view(data: &RunData) -> IdentityView {
    let launcher = data.run.user_id.as_str();
    let run_as = data.agent_run.user_id.as_str();
    let identity = data.identity.as_ref();
    IdentityView {
        client_name: identity.and_then(|i| i.client_name.clone()),
        client_url: identity
            .and_then(|i| i.client_id.as_ref())
            .map(|id| format!("/admin/farm/clients/{id}")),
        launcher_name: identity.map_or_else(|| launcher.to_owned(), |i| i.launcher_name.clone()),
        launcher_url: format!("/admin/users/{launcher}"),
        run_as_name: identity
            .and_then(|i| i.run_as_name.clone())
            .unwrap_or_else(|| run_as.to_owned()),
        run_as_url: format!("/admin/users/{run_as}"),
        run_as_differs: launcher != run_as,
    }
}
