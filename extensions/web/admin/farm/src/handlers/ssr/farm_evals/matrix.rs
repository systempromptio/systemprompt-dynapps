//! The case-by-agent matrix on the evals page, pivoted from the latest
//! decided result per case. Rows are `suite › case` in declaration order,
//! columns are agents; a suite belongs to one agent, so every other column
//! of its rows stays empty. A case with no verdict yet keeps its row with an
//! empty cell, and a verdict for a case the suite no longer declares still
//! shows, after the declared ones, so a removed case never hides its history.

use std::collections::{BTreeSet, HashMap};

use serde::Serialize;

use crate::repositories::rows::{EvalCaseRow, EvalSuiteRow};
use crate::repositories::summaries::MatrixCellRow;
use crate::types::FarmAgentId;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MatrixCellView {
    pub status: String,
    pub passed: bool,
    pub glyph: &'static str,
    pub tone: &'static str,
    pub label: &'static str,
    pub score_display: Option<String>,
    pub score_tone: &'static str,
    pub rationale: String,
    pub href: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MatrixSlotView {
    pub agent_id: FarmAgentId,
    pub cell: Option<MatrixCellView>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MatrixRowView {
    pub suite_id: String,
    pub case_id: String,
    pub slots: Vec<MatrixSlotView>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct EvalMatrixView {
    pub agents: Vec<String>,
    pub rows: Vec<MatrixRowView>,
    pub decided: usize,
    pub passed: usize,
}

#[must_use]
pub fn score_tone(score: f64) -> &'static str {
    if score >= 80.0 {
        "ok"
    } else if score >= 50.0 {
        "warn"
    } else {
        "err"
    }
}

fn verdict(status: &str, passed: bool) -> (&'static str, &'static str, &'static str) {
    match (status, passed) {
        (_, true) => ("✓", "ok", "Passed"),
        ("errored", false) => ("!", "warn", "Errored"),
        _ => ("✗", "err", "Failed"),
    }
}

fn cell_view(row: &MatrixCellRow) -> MatrixCellView {
    let (glyph, tone, label) = verdict(&row.status, row.passed);
    MatrixCellView {
        status: row.status.clone(),
        passed: row.passed,
        glyph,
        tone,
        label,
        score_display: row.judge_score.map(|s| format!("{s:.0}")),
        score_tone: row.judge_score.map_or("muted", score_tone),
        rationale: row.rationale.clone().unwrap_or_default(),
        href: format!("/admin/farm/evals/runs/{}", row.eval_run_id.as_str()),
    }
}

fn row_keys(
    suites: &[EvalSuiteRow],
    cases: &[EvalCaseRow],
    cells: &[MatrixCellRow],
) -> Vec<(String, String)> {
    let mut keys: Vec<(String, String)> = Vec::new();
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut push = |suite: &str, case: &str| {
        let key = (suite.to_owned(), case.to_owned());
        if seen.insert(key.clone()) {
            keys.push(key);
        }
    };
    for suite in suites {
        for case in cases.iter().filter(|c| c.suite_id == suite.id) {
            push(suite.id.as_str(), &case.id);
        }
        for cell in cells.iter().filter(|c| c.suite_id == suite.id) {
            push(suite.id.as_str(), &cell.case_id);
        }
    }
    keys
}

#[must_use]
pub fn eval_matrix(
    suites: &[EvalSuiteRow],
    cases: &[EvalCaseRow],
    cells: &[MatrixCellRow],
) -> EvalMatrixView {
    let agents: Vec<String> = suites
        .iter()
        .map(|s| s.agent_id.as_str().to_owned())
        .chain(
            cells
                .iter()
                .filter(|c| suites.iter().any(|s| s.id == c.suite_id))
                .map(|c| c.agent_id.as_str().to_owned()),
        )
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let by_key: HashMap<(&str, &str, &str), &MatrixCellRow> = cells
        .iter()
        .map(|c| {
            (
                (c.suite_id.as_str(), c.case_id.as_str(), c.agent_id.as_str()),
                c,
            )
        })
        .collect();
    let mut view = EvalMatrixView {
        agents: agents.clone(),
        ..EvalMatrixView::default()
    };
    for (suite_id, case_id) in row_keys(suites, cases, cells) {
        let slots = agents
            .iter()
            .map(|agent| {
                let cell = by_key
                    .get(&(suite_id.as_str(), case_id.as_str(), agent.as_str()))
                    .copied()
                    .map(cell_view);
                if let Some(c) = &cell {
                    view.decided += 1;
                    view.passed += usize::from(c.passed);
                }
                MatrixSlotView {
                    agent_id: FarmAgentId::new(agent.clone()),
                    cell,
                }
            })
            .collect();
        view.rows.push(MatrixRowView {
            suite_id,
            case_id,
            slots,
        });
    }
    view
}
