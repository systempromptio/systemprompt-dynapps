//! The evals page's case-by-agent matrix pivot: row order, empty slots for a
//! case with no verdict, verdict glyphs and score tones, and cases a suite no
//! longer declares.

use chrono::Utc;
use serde_json::json;
use systemprompt_web_admin_farm::repositories::rows::{EvalCaseRow, EvalSuiteRow};
use systemprompt_web_admin_farm::repositories::summaries::MatrixCellRow;
use systemprompt_web_admin_farm::types::{EvalRunId, EvalSuiteId, FarmAgentId, FarmId};
use systemprompt_web_admin_farm::views::eval_matrix::{eval_matrix, score_tone};

fn suite(id: &str, agent: &str) -> EvalSuiteRow {
    EvalSuiteRow {
        id: EvalSuiteId::new(id),
        farm_id: FarmId::new("farm"),
        agent_id: FarmAgentId::new(agent),
        source_path: format!("evals/{id}.yaml"),
        case_count: 2,
        synced_commit_sha: "abc".to_owned(),
        updated_at: Utc::now(),
    }
}

fn case(suite: &str, id: &str) -> EvalCaseRow {
    EvalCaseRow {
        suite_id: EvalSuiteId::new(suite),
        id: id.to_owned(),
        prompt: None,
        fixture: None,
        timeout_secs: None,
        assertions: json!([]),
        judge_rubric: None,
        judge_threshold: None,
    }
}

fn cell(suite: &str, case: &str, agent: &str, status: &str, score: Option<f64>) -> MatrixCellRow {
    MatrixCellRow {
        suite_id: EvalSuiteId::new(suite),
        case_id: case.to_owned(),
        agent_id: FarmAgentId::new(agent),
        eval_run_id: EvalRunId::new("er-1"),
        status: status.to_owned(),
        passed: status == "passed",
        judge_score: score,
        rationale: Some("because".to_owned()),
        finished_at: Some(Utc::now()),
    }
}

#[test]
fn pivots_cases_into_agent_columns_in_declaration_order() {
    let suites = vec![suite("s-review", "reviewer"), suite("s-write", "writer")];
    let cases = vec![
        case("s-review", "b-case"),
        case("s-review", "a-case"),
        case("s-write", "only"),
    ];
    let cells = vec![
        cell("s-review", "b-case", "reviewer", "passed", Some(91.0)),
        cell("s-write", "only", "writer", "failed", Some(40.0)),
    ];
    let view = eval_matrix(&suites, &cases, &cells);

    assert_eq!(view.agents, vec!["reviewer", "writer"]);
    let keys: Vec<(&str, &str)> = view
        .rows
        .iter()
        .map(|r| (r.suite_id.as_str(), r.case_id.as_str()))
        .collect();
    assert_eq!(
        keys,
        vec![
            ("s-review", "b-case"),
            ("s-review", "a-case"),
            ("s-write", "only")
        ]
    );
    assert_eq!(view.decided, 2);
    assert_eq!(view.passed, 1);

    let first = &view.rows[0];
    let hit = first.slots[0].cell.as_ref().unwrap();
    assert_eq!(first.slots[0].agent_id.as_str(), "reviewer");
    assert_eq!((hit.glyph, hit.tone, hit.score_tone), ("✓", "ok", "ok"));
    assert_eq!(hit.score_display.as_deref(), Some("91"));
    assert_eq!(hit.href, "/admin/farm/evals/runs/er-1");
    assert!(first.slots[1].cell.is_none(), "writer column stays empty");

    assert!(
        view.rows[1].slots.iter().all(|s| s.cell.is_none()),
        "no verdict yet"
    );

    let failed = view.rows[2].slots[1].cell.as_ref().unwrap();
    assert_eq!(
        (failed.glyph, failed.tone, failed.score_tone),
        ("✗", "err", "err")
    );
}

#[test]
fn keeps_a_verdict_for_a_removed_case_and_marks_errors() {
    let suites = vec![suite("s", "agent")];
    let cases = vec![case("s", "kept")];
    let cells = vec![cell("s", "gone", "agent", "errored", None)];
    let view = eval_matrix(&suites, &cases, &cells);

    assert_eq!(view.rows.len(), 2);
    assert_eq!(view.rows[1].case_id, "gone");
    let errored = view.rows[1].slots[0].cell.as_ref().unwrap();
    assert_eq!((errored.glyph, errored.tone), ("!", "warn"));
    assert_eq!(errored.score_display, None);
    assert_eq!(errored.score_tone, "muted");
}

#[test]
fn filters_cells_to_the_suites_given() {
    let view = eval_matrix(
        &[suite("mine", "a")],
        &[],
        &[cell("other", "x", "b", "passed", Some(100.0))],
    );
    assert_eq!(view.agents, vec!["a"]);
    assert!(view.rows.is_empty());
    assert_eq!(view.decided, 0);
}

#[test]
fn score_tone_uses_the_ai_score_bands() {
    assert_eq!(score_tone(80.0), "ok");
    assert_eq!(score_tone(79.9), "warn");
    assert_eq!(score_tone(50.0), "warn");
    assert_eq!(score_tone(49.0), "err");
}
