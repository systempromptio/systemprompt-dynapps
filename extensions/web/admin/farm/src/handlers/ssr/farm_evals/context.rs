//! The evals page's view model: the farm and window filters, the KPI strip,
//! and one card per suite with its last eval run and a score sparkline.

use chrono::{DateTime, Utc};
use serde::Serialize;
use systemprompt_web_admin_core::ssr::format::relative_time;
use systemprompt_web_admin_core::ssr::tone::score_display;
use systemprompt_web_admin_core::ssr::types::sparkline::{SparklineView, sparkline_toned};
use systemprompt_web_shared::format::format_cost;

use super::matrix::score_tone;
use crate::handlers::ssr::shared::BadgeView;
use crate::repositories::rows::{EvalRunRow, EvalSuiteRow, FarmRow};
use crate::types::{EvalRunStatus, FarmAgentId};

pub(crate) const WINDOWS: [u32; 3] = [7, 30, 90];
const SPARK_RUNS: usize = 12;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OptionView {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct KpiView {
    pub key: &'static str,
    pub label: &'static str,
    pub value: String,
    pub tone: &'static str,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LastRunView {
    pub status: BadgeView,
    pub href: String,
    pub relative: String,
    pub score_display: String,
    pub score_tone: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct EvalSuiteRowView {
    pub suite_id: String,
    pub farm_id: String,
    pub agent_id: FarmAgentId,
    pub agent_url: String,
    pub source_path: String,
    pub case_count: i32,
    pub run_url: String,
    pub last_run: Option<LastRunView>,
    pub spark: SparklineView,
}

pub(crate) fn eval_status_badge(status: &str) -> BadgeView {
    let (tone, text) = match EvalRunStatus::parse(status) {
        Some(EvalRunStatus::Queued) => ("muted", "Queued"),
        Some(EvalRunStatus::Running) => ("accent", "Running"),
        Some(EvalRunStatus::Completed) => ("ok", "Completed"),
        Some(EvalRunStatus::Failed) => ("err", "Failed"),
        Some(EvalRunStatus::Cancelled) => ("muted", "Cancelled"),
        None => ("muted", "Unknown"),
    };
    BadgeView::new(tone, text, status)
}

pub(crate) fn is_eval_live(status: &str) -> bool {
    matches!(
        EvalRunStatus::parse(status),
        Some(EvalRunStatus::Queued | EvalRunStatus::Running)
    )
}

pub(crate) fn score_cell(score: Option<f64>) -> (String, &'static str) {
    (score_display(score), score.map_or("muted", score_tone))
}

pub(crate) fn farm_options(farms: &[FarmRow], selected: Option<&str>) -> Vec<OptionView> {
    farms
        .iter()
        .map(|f| OptionView {
            value: f.id.as_str().to_owned(),
            label: f.name.clone(),
            selected: selected == Some(f.id.as_str()),
        })
        .collect()
}

pub(crate) fn window_options(days: u32) -> Vec<OptionView> {
    WINDOWS
        .iter()
        .map(|d| OptionView {
            value: d.to_string(),
            label: format!("{d} days"),
            selected: *d == days,
        })
        .collect()
}

pub(crate) fn in_window(runs: Vec<EvalRunRow>, since: DateTime<Utc>) -> Vec<EvalRunRow> {
    runs.into_iter().filter(|r| r.created_at >= since).collect()
}

fn ratio_display(num: i64, den: i64) -> (String, Option<f64>) {
    if den == 0 {
        return ("—".to_owned(), None);
    }
    let pct = num as f64 * 100.0 / den as f64;
    (format!("{pct:.0}%"), Some(pct))
}

pub(crate) fn kpis(suites: &[EvalSuiteRow], runs: &[EvalRunRow], days: u32) -> Vec<KpiView> {
    let cases: i64 = suites.iter().map(|s| i64::from(s.case_count)).sum();
    let scores: Vec<f64> = runs.iter().filter_map(|r| r.mean_score).collect();
    let mean = (!scores.is_empty()).then(|| scores.iter().sum::<f64>() / scores.len() as f64);
    let (mean_value, mean_tone) = score_cell(mean);
    let passed: i64 = runs.iter().map(|r| i64::from(r.passed)).sum();
    let decided: i64 = runs
        .iter()
        .map(|r| i64::from(r.passed + r.failed + r.errored))
        .sum();
    let (rate_value, rate) = ratio_display(passed, decided);
    let cost: i64 = runs.iter().map(|r| r.cost_microdollars).sum();
    let window = format!("last {days} days");
    vec![
        KpiView {
            key: "suites",
            label: "Suites",
            value: suites.len().to_string(),
            tone: "info",
            note: "synced from eval files".to_owned(),
        },
        KpiView {
            key: "cases",
            label: "Cases",
            value: cases.to_string(),
            tone: "info",
            note: "across every suite".to_owned(),
        },
        KpiView {
            key: "runs",
            label: "Eval runs",
            value: runs.len().to_string(),
            tone: "accent",
            note: window.clone(),
        },
        KpiView {
            key: "score",
            label: "Mean judge score",
            value: mean_value,
            tone: mean_tone,
            note: format!("{} judged runs", scores.len()),
        },
        KpiView {
            key: "pass",
            label: "Pass rate",
            value: rate_value,
            tone: rate.map_or("muted", score_tone),
            note: format!("{passed} of {decided} cases"),
        },
        KpiView {
            key: "cost",
            label: "Cost",
            value: format_cost(cost),
            tone: "muted",
            note: window,
        },
    ]
}

fn last_run(run: &EvalRunRow) -> LastRunView {
    let (score_display, score_tone) = score_cell(run.mean_score);
    LastRunView {
        status: eval_status_badge(&run.status),
        href: format!("/admin/farm/evals/runs/{}", run.id.as_str()),
        relative: relative_time(run.created_at),
        score_display,
        score_tone,
    }
}

pub(crate) fn suite_views(suites: &[EvalSuiteRow], runs: &[EvalRunRow]) -> Vec<EvalSuiteRowView> {
    suites
        .iter()
        .map(|suite| {
            let mine: Vec<&EvalRunRow> = runs.iter().filter(|r| r.suite_id == suite.id).collect();
            let mut scores: Vec<i64> = mine
                .iter()
                .take(SPARK_RUNS)
                .filter_map(|r| r.mean_score.map(|s| s.round() as i64))
                .collect();
            scores.reverse();
            let tone = scores.last().map_or("muted", |s| score_tone(*s as f64));
            EvalSuiteRowView {
                suite_id: suite.id.as_str().to_owned(),
                farm_id: suite.farm_id.as_str().to_owned(),
                agent_id: suite.agent_id.clone(),
                agent_url: format!(
                    "/admin/farm/{}/agents/{}",
                    suite.farm_id.as_str(),
                    suite.agent_id.as_str()
                ),
                source_path: suite.source_path.clone(),
                case_count: suite.case_count,
                run_url: format!("/admin/farm/evals/{}/run", suite.id.as_str()),
                last_run: mine.first().copied().map(last_run),
                spark: sparkline_toned(&scores, tone, "Mean judge score, latest runs".to_owned()),
            }
        })
        .collect()
}
