//! The eval history pieces the evals page and the agent page share: one
//! trend row per agent (pass rate and judge score sparklines over the
//! window's days, with the window's totals) and the recent eval runs table,
//! each run with the surface that triggered it.

use std::collections::BTreeMap;

use serde::Serialize;
use systemprompt_web_admin_core::ssr::format::relative_time;
use systemprompt_web_admin_core::ssr::types::sparkline::{SparklineView, sparkline_toned};
use systemprompt_web_shared::format::format_cost;

use super::status::{BadgeView, trigger_badge};
use crate::handlers::ssr::farm_evals::matrix::score_tone;
use crate::handlers::ssr::farm_evals::{eval_status_badge, score_cell};
use crate::repositories::rows::{EvalDayRow, EvalRunRow};
use crate::types::FarmAgentId;

#[derive(Debug, Serialize)]
pub struct EvalTrendRowView {
    pub farm_id: String,
    pub agent_id: FarmAgentId,
    pub agent_url: String,
    pub runs: i64,
    pub days: usize,
    pub pass_display: String,
    pub pass_tone: &'static str,
    pub score_display: String,
    pub score_tone: &'static str,
    pub cost: String,
    pub pass_spark: SparklineView,
    pub score_spark: SparklineView,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EvalRunItemView {
    pub href: String,
    pub relative: String,
    pub suite_id: String,
    pub agent_id: FarmAgentId,
    pub trigger: BadgeView,
    pub status: BadgeView,
    pub passed: i32,
    pub failed: i32,
    pub errored: i32,
    pub score_display: String,
    pub score_tone: &'static str,
    pub cost: String,
}

fn pct(passed: i64, decided: i64) -> Option<f64> {
    (decided > 0).then(|| passed as f64 * 100.0 / decided as f64)
}

fn trend_row(days: &[&EvalDayRow]) -> Option<EvalTrendRowView> {
    let first = days.first()?;
    let runs: i64 = days.iter().map(|d| d.runs).sum();
    let passed: i64 = days.iter().map(|d| d.passed).sum();
    let decided: i64 = days.iter().map(|d| d.decided).sum();
    let scored: Vec<(f64, i64)> = days
        .iter()
        .filter_map(|d| d.mean_score.map(|s| (s, d.runs)))
        .collect();
    let weight: i64 = scored.iter().map(|(_, n)| n).sum();
    let mean = (weight > 0)
        .then(|| scored.iter().map(|(s, n)| s * *n as f64).sum::<f64>() / weight as f64);
    let rate = pct(passed, decided);
    let (pass_display, pass_tone) = rate.map_or_else(
        || ("—".to_owned(), "muted"),
        |r| (format!("{r:.0}%"), score_tone(r)),
    );
    let (score_text, mean_tone) = score_cell(mean);
    let pass_values: Vec<i64> = days
        .iter()
        .filter_map(|d| pct(d.passed, d.decided).map(|r| r.round() as i64))
        .collect();
    let score_values: Vec<i64> = days
        .iter()
        .filter_map(|d| d.mean_score.map(|s| s.round() as i64))
        .collect();
    Some(EvalTrendRowView {
        farm_id: first.farm_id.as_str().to_owned(),
        agent_id: first.agent_id.clone(),
        agent_url: format!(
            "/admin/farm/{}/agents/{}",
            first.farm_id.as_str(),
            first.agent_id.as_str()
        ),
        runs,
        days: days.len(),
        pass_display,
        pass_tone,
        score_display: score_text,
        score_tone: mean_tone,
        cost: format_cost(days.iter().map(|d| d.cost_microdollars).sum()),
        pass_spark: sparkline_toned(&pass_values, pass_tone, "Pass rate per day".to_owned()),
        score_spark: sparkline_toned(
            &score_values,
            mean_tone,
            "Mean judge score per day".to_owned(),
        ),
    })
}

#[must_use]
pub fn eval_trend_rows(history: &[EvalDayRow]) -> Vec<EvalTrendRowView> {
    let mut by_agent: BTreeMap<(&str, &str), Vec<&EvalDayRow>> = BTreeMap::new();
    for row in history {
        by_agent
            .entry((row.farm_id.as_str(), row.agent_id.as_str()))
            .or_default()
            .push(row);
    }
    by_agent
        .into_values()
        .filter_map(|mut days| {
            days.sort_by_key(|d| d.day);
            trend_row(&days)
        })
        .collect()
}

#[must_use]
pub fn eval_run_items(runs: &[EvalRunRow]) -> Vec<EvalRunItemView> {
    runs.iter()
        .map(|run| {
            let (score_text, run_tone) = score_cell(run.mean_score);
            EvalRunItemView {
                href: format!("/admin/farm/evals/runs/{}", run.id.as_str()),
                relative: relative_time(run.created_at),
                suite_id: run.suite_id.as_str().to_owned(),
                agent_id: run.agent_id.clone(),
                trigger: trigger_badge(&run.triggered_via),
                status: eval_status_badge(&run.status),
                passed: run.passed,
                failed: run.failed,
                errored: run.errored,
                score_display: score_text,
                score_tone: run_tone,
                cost: format_cost(run.cost_microdollars),
            }
        })
        .collect()
}
