//! The leaderboard row view: one person's runs, their share of the window,
//! outcome counts, cost and how their runs were triggered.

use serde::Serialize;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::ssr::format::{local_time, relative_time};
use systemprompt_web_shared::format::format_cost;

use crate::repositories::rows::RunsByUserRow;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct TriggerSplit {
    pub dashboard: f64,
    pub api: f64,
    pub eval: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PersonRowView {
    pub user_id: UserId,
    pub display_name: String,
    pub user_url: String,
    pub runs_url: String,
    pub runs: i64,
    pub share_pct: f64,
    pub share_label: String,
    pub succeeded: i64,
    pub failed: i64,
    pub active: i64,
    pub success_label: String,
    pub cost_display: String,
    pub favourite_agent: String,
    pub split: TriggerSplit,
    pub split_title: String,
    pub dashboard_runs: i64,
    pub api_runs: i64,
    pub eval_runs: i64,
    pub last_run_relative: String,
    pub last_run_local: String,
}

#[must_use]
pub fn share_pct(part: i64, total: i64) -> f64 {
    if total <= 0 || part <= 0 {
        return 0.0;
    }
    let pct = (part as f64) * 100.0 / (total as f64);
    (pct.min(100.0) * 10.0).round() / 10.0
}

#[must_use]
pub fn trigger_split(dashboard: i64, api: i64, eval: i64) -> TriggerSplit {
    let total = dashboard.max(0) + api.max(0) + eval.max(0);
    if total == 0 {
        return TriggerSplit {
            dashboard: 0.0,
            api: 0.0,
            eval: 0.0,
        };
    }
    let dashboard = share_pct(dashboard, total);
    let api = share_pct(api, total);
    let eval = if eval > 0 {
        ((100.0 - dashboard - api).max(0.0) * 10.0).round() / 10.0
    } else {
        0.0
    };
    TriggerSplit {
        dashboard,
        api,
        eval,
    }
}

#[must_use]
pub fn success_label(succeeded: i64, failed: i64) -> String {
    let finished = succeeded + failed;
    if finished <= 0 {
        return "—".to_owned();
    }
    format!("{}%", share_pct(succeeded, finished).round())
}

#[must_use]
pub fn total_runs(rows: &[RunsByUserRow]) -> i64 {
    rows.iter().map(|r| r.runs.unwrap_or(0)).sum()
}

#[must_use]
pub(crate) fn person_row(row: &RunsByUserRow, window_total: i64) -> PersonRowView {
    let runs = row.runs.unwrap_or(0);
    let succeeded = row.succeeded.unwrap_or(0);
    let failed = row.failed.unwrap_or(0);
    let dashboard_runs = row.dashboard_runs.unwrap_or(0);
    let api_runs = row.api_runs.unwrap_or(0);
    let eval_runs = row.eval_runs.unwrap_or(0);
    let share = share_pct(runs, window_total);
    let id = row.user_id.as_str();
    PersonRowView {
        user_id: row.user_id.clone(),
        display_name: row
            .display_name
            .clone()
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| id.to_owned()),
        user_url: format!("/admin/users/{id}"),
        runs_url: format!("/admin/farm/runs?user_id={id}"),
        runs,
        share_pct: share,
        share_label: format!("{share}%"),
        succeeded,
        failed,
        active: row.active.unwrap_or(0),
        success_label: success_label(succeeded, failed),
        cost_display: format_cost(row.cost_microdollars.unwrap_or(0)),
        favourite_agent: row.favourite_agent.clone().unwrap_or_default(),
        split: trigger_split(dashboard_runs, api_runs, eval_runs),
        split_title: format!("Dashboard {dashboard_runs} · API {api_runs} · Eval {eval_runs}"),
        dashboard_runs,
        api_runs,
        eval_runs,
        last_run_relative: row.last_run_at.map(relative_time).unwrap_or_default(),
        last_run_local: row.last_run_at.map(local_time).unwrap_or_default(),
    }
}

#[must_use]
pub fn person_rows(rows: &[RunsByUserRow]) -> Vec<PersonRowView> {
    let total = total_runs(rows);
    rows.iter().map(|r| person_row(r, total)).collect()
}
