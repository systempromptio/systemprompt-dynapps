//! The live half of the eval run page: the KPI strip and the per-case rows.
//! The page embeds it and the fragment returns it, both from
//! [`eval_run_live`], so a figure reads the same before and after a poll.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use systemprompt_web_shared::format::{format_cost, format_duration_ms, short_id};

use crate::handlers::ssr::farm_evals::{eval_status_badge, is_eval_live, score_cell};
use crate::handlers::ssr::shared::{BadgeView, status_badge};
use crate::repositories::rows::{EvalResultRow, EvalRunRow};
use crate::types::EvalResultStatus;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LiveKpiView {
    pub key: &'static str,
    pub label: &'static str,
    pub value: String,
    pub tone: &'static str,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub(crate) struct AssertionView {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub passed: bool,
    #[serde(default)]
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CaseRowView {
    pub case_id: String,
    pub detail_id: String,
    pub status: BadgeView,
    pub assertions: Vec<AssertionView>,
    pub assertions_display: String,
    pub score_display: String,
    pub score_tone: &'static str,
    pub has_score: bool,
    pub rationale: String,
    pub run_url: Option<String>,
    pub run_id_short: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EvalRunLiveView {
    pub fragment_url: String,
    pub is_live: bool,
    pub status: BadgeView,
    pub kpis: Vec<LiveKpiView>,
    pub rows: Vec<CaseRowView>,
}

fn result_badge(status: &str) -> BadgeView {
    let (tone, text) = match EvalResultStatus::parse(status) {
        Some(EvalResultStatus::Pending) => ("muted", "Pending"),
        Some(EvalResultStatus::Running) => ("accent", "Running"),
        Some(EvalResultStatus::Passed) => ("ok", "Passed"),
        Some(EvalResultStatus::Failed) => ("err", "Failed"),
        Some(EvalResultStatus::Errored) => ("warn", "Errored"),
        None => return status_badge(status),
    };
    BadgeView::new(tone, text, status)
}

fn case_row(result: &EvalResultRow) -> CaseRowView {
    let assertions = Vec::<AssertionView>::deserialize(&result.assertions).unwrap_or_default();
    let held = assertions.iter().filter(|a| a.passed).count();
    let (score_display, score_tone) = score_cell(result.judge_score);
    CaseRowView {
        case_id: result.case_id.clone(),
        detail_id: format!("eval-case-{}", result.id.as_str()),
        status: result_badge(&result.status),
        assertions_display: format!("{held} / {}", assertions.len()),
        assertions,
        score_display,
        score_tone,
        has_score: result.judge_score.is_some(),
        rationale: result.judge_rationale.clone().unwrap_or_default(),
        run_url: result
            .run_id
            .as_ref()
            .map(|id| format!("/admin/farm/runs/{id}")),
        run_id_short: result.run_id.as_deref().map(short_id).unwrap_or_default(),
    }
}

fn duration_display(run: &EvalRunRow) -> String {
    let Some(started) = run.started_at else {
        return "—".to_owned();
    };
    let end = run.finished_at.unwrap_or_else(Utc::now);
    format_duration_ms((end - started).num_milliseconds().max(0))
}

fn kpis(run: &EvalRunRow, results: &[EvalResultRow]) -> Vec<LiveKpiView> {
    let (score, score_tone) = score_cell(run.mean_score);
    let kpi = |key, label, value: String, tone| LiveKpiView {
        key,
        label,
        value,
        tone,
    };
    vec![
        kpi("cases", "Cases", results.len().to_string(), "info"),
        kpi("passed", "Passed", run.passed.to_string(), "ok"),
        kpi(
            "failed",
            "Failed",
            (run.failed + run.errored).to_string(),
            if run.failed + run.errored > 0 {
                "err"
            } else {
                "muted"
            },
        ),
        kpi("score", "Mean score", score, score_tone),
        kpi("cost", "Cost", format_cost(run.cost_microdollars), "muted"),
        kpi("duration", "Duration", duration_display(run), "muted"),
    ]
}

pub(crate) fn eval_run_live(run: &EvalRunRow, results: &[EvalResultRow]) -> EvalRunLiveView {
    EvalRunLiveView {
        fragment_url: format!("/admin/api/farm/evals/runs/{}", run.id.as_str()),
        is_live: is_eval_live(&run.status),
        status: eval_status_badge(&run.status),
        kpis: kpis(run, results),
        rows: results.iter().map(case_row).collect(),
    }
}
