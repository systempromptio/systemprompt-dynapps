//! The agent page's read-only pieces: the header meta, the KPI strip over the
//! last thirty days, the definition card and the eval mini-matrix.

use serde::Serialize;
use systemprompt::identifiers::PluginId;
use systemprompt_web_shared::format::{format_cost, format_duration_ms, short_id};

use crate::repositories::rows::{AgentRow, EvalCaseRow, FarmRow, RunRow};
use crate::repositories::summaries::{MatrixCellRow, RunStats};
use crate::types::RunStatus;

const DASH: &str = "—";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HeaderView {
    pub farm_name: String,
    pub farm_url: String,
    pub model: String,
    pub plugin_id: PluginId,
    pub fixture: Option<String>,
    pub synced_sha_short: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AgentKpiView {
    pub key: &'static str,
    pub label: &'static str,
    pub value: String,
    pub note: Option<String>,
    pub tone: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DefinitionView {
    pub description: String,
    pub entry_prompt: String,
    pub tools: Vec<String>,
    pub skills: Vec<String>,
    pub source_path: String,
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CaseCellView {
    pub case_id: String,
    pub tone: &'static str,
    pub verdict: &'static str,
    pub score: String,
    pub rationale: Option<String>,
    pub eval_run_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MatrixView {
    pub suite_id: String,
    pub run_url: String,
    pub farm_id: String,
    pub cases: Vec<CaseCellView>,
}

pub(crate) fn header_view(farm: &FarmRow, agent: &AgentRow) -> HeaderView {
    HeaderView {
        farm_name: farm.name.clone(),
        farm_url: format!("/admin/farm/{}", farm.id.as_str()),
        model: agent
            .model
            .clone()
            .or_else(|| farm.model.clone())
            .unwrap_or_else(|| "default".to_owned()),
        plugin_id: agent.plugin_id.clone(),
        fixture: agent.fixture.clone(),
        synced_sha_short: short_id(&agent.synced_commit_sha),
    }
}

const fn kpi(key: &'static str, label: &'static str, value: String) -> AgentKpiView {
    AgentKpiView {
        key,
        label,
        value,
        note: None,
        tone: None,
    }
}

fn mean_turns(runs: &[RunRow]) -> Option<f64> {
    let turns: Vec<i32> = runs.iter().filter_map(|r| r.num_turns).collect();
    let count = i32::try_from(turns.len()).ok().filter(|n| *n > 0)?;
    Some(f64::from(turns.iter().sum::<i32>()) / f64::from(count))
}

fn mean_score(cells: &[MatrixCellRow]) -> Option<f64> {
    let scores: Vec<f64> = cells.iter().filter_map(|c| c.judge_score).collect();
    let count = u32::try_from(scores.len()).ok().filter(|n| *n > 0)?;
    Some(scores.iter().sum::<f64>() / f64::from(count))
}

fn duration(ms: Option<f64>) -> String {
    ms.map_or_else(|| DASH.to_owned(), |v| format_duration_ms(v.round() as i64))
}

pub(crate) fn kpi_views(
    stats: &RunStats,
    recent: &[RunRow],
    cells: &[MatrixCellRow],
) -> Vec<AgentKpiView> {
    let decided = stats.succeeded + stats.failed + stats.timed_out;
    let finished = decided + stats.cancelled;
    let mut success = kpi(
        "success",
        "Success",
        if decided > 0 {
            format!("{}%", stats.succeeded * 100 / decided)
        } else {
            DASH.to_owned()
        },
    );
    success.note = Some(format!("{} of {decided} decided", stats.succeeded));
    success.tone = (stats.failed + stats.timed_out > 0).then_some("warn");
    let mut runs = kpi("runs", "Runs (30d)", stats.total.to_string());
    runs.note = (stats.running + stats.queued > 0)
        .then(|| format!("{} live", stats.running + stats.queued));
    let mean_cost = if finished > 0 {
        format_cost(stats.cost_microdollars / finished)
    } else {
        DASH.to_owned()
    };
    let mut latency = kpi("p50", "Duration p50", duration(stats.p50_duration_ms));
    latency.note = Some(format!("p95 {}", duration(stats.p95_duration_ms)));
    vec![
        runs,
        success,
        latency,
        kpi("cost", "Mean cost", mean_cost),
        kpi(
            "turns",
            "Mean turns",
            mean_turns(recent).map_or_else(|| DASH.to_owned(), |t| format!("{t:.1}")),
        ),
        kpi(
            "judge",
            "Mean judge score",
            mean_score(cells).map_or_else(|| DASH.to_owned(), |s| format!("{s:.1}")),
        ),
    ]
}

pub(crate) fn definition_view(agent: &AgentRow) -> DefinitionView {
    DefinitionView {
        description: agent.description.clone(),
        entry_prompt: agent.entry_prompt.clone(),
        tools: agent.tools.clone(),
        skills: agent.skills.clone(),
        source_path: agent.source_path.clone(),
        output_path: agent.output_path.clone(),
    }
}

fn case_cell(case: &EvalCaseRow, cell: Option<&MatrixCellRow>) -> CaseCellView {
    let (tone, verdict) = match cell.map(|c| c.status.as_str()) {
        Some("passed") => ("ok", "Passed"),
        Some("failed") => ("err", "Failed"),
        Some("errored") => ("warn", "Errored"),
        _ => ("muted", "Not run"),
    };
    CaseCellView {
        case_id: case.id.clone(),
        tone,
        verdict,
        score: cell
            .and_then(|c| c.judge_score)
            .map_or_else(|| DASH.to_owned(), |s| format!("{s:.1}")),
        rationale: cell.and_then(|c| c.rationale.clone()),
        eval_run_url: cell.map(|c| format!("/admin/farm/evals/runs/{}", c.eval_run_id.as_str())),
    }
}

pub(crate) fn matrix_view(
    farm: &FarmRow,
    suite_id: &str,
    cases: &[EvalCaseRow],
    cells: &[MatrixCellRow],
) -> Option<MatrixView> {
    if cases.is_empty() {
        return None;
    }
    Some(MatrixView {
        suite_id: suite_id.to_owned(),
        run_url: format!("/admin/farm/evals/{suite_id}/run"),
        farm_id: farm.id.as_str().to_owned(),
        cases: cases
            .iter()
            .map(|case| case_cell(case, cells.iter().find(|c| c.case_id == case.id)))
            .collect(),
    })
}

pub(crate) fn is_finished(run: &RunRow) -> bool {
    RunStatus::parse(&run.status).is_some_and(RunStatus::is_terminal)
}
