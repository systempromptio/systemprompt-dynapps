//! The "My routines" page's view model: one row per live routine, with its
//! latest runs folded beneath it.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Serialize;
// JSON: protocol boundary — reads `name`, `enabled` and `schedule` from
// DynWork's definition.
use serde_json::Value;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

use super::PAGE_URL;
use crate::ids::RunState;
use crate::store::{RoutineRecord, RunRecord};

#[derive(Debug, Serialize)]
pub(super) struct RoutinesPageView {
    page: &'static str,
    title: &'static str,
    breadcrumbs: Vec<BreadcrumbView>,
    count_label: String,
    has_rows: bool,
    rows: Vec<RoutineRowView>,
}

#[derive(Debug, Serialize)]
struct RoutineRowView {
    name: String,
    schedule: String,
    computer: String,
    enabled: bool,
    status_label: &'static str,
    status_tone: &'static str,
    last_run: Option<RunCellView>,
    runs: Vec<RunCellView>,
    has_runs: bool,
    pause_url: String,
    resume_url: String,
    delete_url: String,
}

#[derive(Debug, Serialize)]
struct RunCellView {
    label: &'static str,
    tone: &'static str,
    when: String,
    reason: Option<String>,
}

pub(super) fn page(routines: Vec<RoutineRecord>, runs: &[RunRecord]) -> RoutinesPageView {
    let mut by_routine: HashMap<&str, Vec<&RunRecord>> = HashMap::new();
    for run in runs {
        by_routine
            .entry(run.routine_id.as_str())
            .or_default()
            .push(run);
    }
    let rows: Vec<RoutineRowView> = routines
        .into_iter()
        .map(|routine| {
            let runs = by_routine.remove(routine.id.as_str()).unwrap_or_default();
            row(routine, &runs)
        })
        .collect();
    RoutinesPageView {
        page: "routines",
        title: "My routines",
        breadcrumbs: vec![
            BreadcrumbView::link("Account", "/admin/profile"),
            BreadcrumbView::current("My routines"),
        ],
        count_label: format!("{} routines", rows.len()),
        has_rows: !rows.is_empty(),
        rows,
    }
}

fn row(routine: RoutineRecord, runs: &[&RunRecord]) -> RoutineRowView {
    let definition = &routine.definition;
    let enabled = definition
        .get("enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let base = format!("{PAGE_URL}/{}", routine.id);
    let cells: Vec<RunCellView> = runs.iter().map(|run| cell(run)).collect();
    RoutineRowView {
        name: definition
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or(routine.id.as_str())
            .to_owned(),
        schedule: schedule(definition),
        computer: if routine.owner_name.is_empty() {
            routine.owner_device
        } else {
            routine.owner_name
        },
        enabled,
        status_label: if enabled { "On" } else { "Paused" },
        status_tone: if enabled { "ok" } else { "muted" },
        last_run: runs.first().map(|run| cell(run)),
        has_runs: !cells.is_empty(),
        runs: cells,
        pause_url: format!("{base}/pause"),
        resume_url: format!("{base}/resume"),
        delete_url: format!("{base}/delete"),
    }
}

fn schedule(definition: &Value) -> String {
    let Some(schedule) = definition.get("schedule") else {
        return "Manual".to_owned();
    };
    let cron = schedule.get("cron").and_then(Value::as_str).unwrap_or("");
    match schedule.get("timezone").and_then(Value::as_str) {
        Some(zone) if !cron.is_empty() => format!("{cron} ({zone})"),
        _ if !cron.is_empty() => cron.to_owned(),
        _ => "Manual".to_owned(),
    }
}

fn cell(run: &RunRecord) -> RunCellView {
    let (label, tone) = state_badge(run.state);
    RunCellView {
        label,
        tone,
        when: stamp(run.started_at.unwrap_or(run.updated_at)),
        reason: run.reason.clone(),
    }
}

const fn state_badge(state: RunState) -> (&'static str, &'static str) {
    match state {
        RunState::Queued => ("Queued", "info"),
        RunState::Running => ("Running", "info"),
        RunState::Ok => ("OK", "ok"),
        RunState::Late => ("Late", "warn"),
        RunState::Failed => ("Failed", "err"),
        RunState::Blocked => ("Blocked", "err"),
        RunState::Missed => ("Missed", "warn"),
        RunState::Skipped => ("Skipped", "muted"),
        RunState::Cancelled => ("Cancelled", "muted"),
    }
}

fn stamp(at: DateTime<Utc>) -> String {
    at.format("%Y-%m-%d %H:%M UTC").to_string()
}
