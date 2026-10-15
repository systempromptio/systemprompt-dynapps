//! The Report tab and the identity badges. The report is the observer's
//! `agent_farm_run_reports` row: whether the task completed, its summary and
//! final message, the files it wrote and the Odoo writes its MCP calls made.
//! The badges name the client the run was for, the person who launched it and
//! the identity its credentials were minted for (core's `agent_runs.user_id`).

use serde::Serialize;
// JSON: the report row's outputs and odoo_writes are JSONB the observer
// writes; each item is read field by field below.
use serde_json::Value;
use systemprompt_web_admin_core::ssr::format::{format_bytes, local_time};

use super::RunData;
use crate::handlers::ssr::farm_clients::views::{completion_badge, eval_badge};
use crate::handlers::ssr::shared::BadgeView;

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
pub(crate) struct OdooWriteView {
    pub tool: String,
    pub model: String,
    pub record_ids: String,
    pub decision: BadgeView,
    pub at: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RunReportView {
    pub has_report: bool,
    pub completion: BadgeView,
    pub eval: BadgeView,
    pub summary: String,
    pub final_message: String,
    pub recorded_local: String,
    pub outputs: Vec<OutputView>,
    pub has_outputs: bool,
    pub odoo_writes: Vec<OdooWriteView>,
    pub has_odoo_writes: bool,
}

fn text(item: &Value, key: &str) -> String {
    match item.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Null) | None => String::new(),
        Some(other) => other.to_string(),
    }
}

fn decision_badge(decision: &str) -> BadgeView {
    let tone = match decision {
        "allow" | "allowed" => "ok",
        "warn" => "warn",
        "deny" | "denied" | "refused" => "err",
        _ => "muted",
    };
    let label = if decision.is_empty() { "—" } else { decision };
    BadgeView::new(tone, label, decision)
}

fn items(value: &Value) -> &[Value] {
    value.as_array().map_or(&[], Vec::as_slice)
}

pub(crate) fn report_view(data: &RunData) -> RunReportView {
    let Some(report) = &data.report else {
        return RunReportView {
            has_report: false,
            completion: completion_badge(None),
            eval: eval_badge(None, None),
            summary: String::new(),
            final_message: String::new(),
            recorded_local: String::new(),
            outputs: Vec::new(),
            has_outputs: false,
            odoo_writes: Vec::new(),
            has_odoo_writes: false,
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
    let odoo_writes: Vec<OdooWriteView> = items(&report.odoo_writes)
        .iter()
        .map(|w| OdooWriteView {
            tool: text(w, "tool"),
            model: text(w, "model"),
            record_ids: match w.get("record_ids") {
                Some(Value::Array(ids)) => ids
                    .iter()
                    .map(|v| v.as_str().map_or_else(|| v.to_string(), str::to_owned))
                    .collect::<Vec<_>>()
                    .join(", "),
                _ => text(w, "record_ids"),
            },
            decision: decision_badge(&text(w, "decision")),
            at: text(w, "at"),
        })
        .collect();
    RunReportView {
        has_report: true,
        completion: completion_badge(Some(report.completed)),
        eval: eval_badge(report.eval_passed, report.eval_score),
        summary: report.summary.clone().unwrap_or_default(),
        final_message: report.final_message.clone().unwrap_or_default(),
        recorded_local: local_time(report.created_at),
        has_outputs: !outputs.is_empty(),
        outputs,
        has_odoo_writes: !odoo_writes.is_empty(),
        odoo_writes,
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
