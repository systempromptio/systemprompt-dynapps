//! Boot-time eval resume: which eval runs a restart left driverless, and
//! what happens to each of their open cases.

use chrono::{DateTime, Duration, TimeZone, Utc};
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_farm::repositories::rows::{EvalResultRow, EvalRunRow};
use systemprompt_web_admin_farm::services::evals::resume::{
    MAX_EVAL_AGE, ResumeAction, plan_resume, select_interrupted,
};
use systemprompt_web_admin_farm::types::{
    EvalResultId, EvalRunId, EvalSuiteId, FarmAgentId, FarmId,
};

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 15, 12, 0, 0)
        .single()
        .expect("valid instant")
}

fn eval_run(status: &str, age: Duration) -> EvalRunRow {
    let created = now() - age;
    EvalRunRow {
        id: EvalRunId::new(format!("er-{status}")),
        suite_id: EvalSuiteId::new("s"),
        farm_id: FarmId::new("f"),
        agent_id: FarmAgentId::new("a"),
        user_id: UserId::new("u@example.com"),
        status: status.to_owned(),
        started_at: Some(created),
        finished_at: None,
        passed: 0,
        failed: 0,
        errored: 0,
        mean_score: None,
        cost_microdollars: 0,
        created_at: created,
        triggered_via: "eval".to_owned(),
    }
}

fn result(id: &str, status: &str, run_id: Option<&str>) -> EvalResultRow {
    EvalResultRow {
        id: EvalResultId::new(id),
        eval_run_id: EvalRunId::new("er"),
        case_id: format!("case-{id}"),
        run_id: run_id.map(str::to_owned),
        status: status.to_owned(),
        assertions: serde_json::json!([]),
        judge_score: None,
        judge_rationale: None,
        created_at: now(),
        updated_at: now(),
    }
}

#[test]
fn only_queued_and_running_eval_runs_are_interrupted() {
    let rows = [
        eval_run("running", Duration::minutes(1)),
        eval_run("queued", Duration::minutes(1)),
        eval_run("completed", Duration::minutes(1)),
        eval_run("failed", Duration::minutes(1)),
        eval_run("cancelled", Duration::minutes(1)),
    ];
    let picked: Vec<&str> = select_interrupted(&rows)
        .iter()
        .map(|r| r.status.as_str())
        .collect();
    assert_eq!(picked, ["running", "queued"]);
}

#[test]
fn open_cases_settle_or_launch_and_decided_ones_are_left() {
    let run = eval_run("running", Duration::minutes(10));
    let results = [
        result("1", "running", Some("run-1")),
        result("2", "pending", None),
        result("3", "passed", Some("run-3")),
        result("4", "errored", Some("run-4")),
    ];
    assert_eq!(
        plan_resume(&run, &results, now()),
        vec![
            ResumeAction::Settle {
                result_id: "1".to_owned(),
                run_id: "run-1".to_owned(),
            },
            ResumeAction::Launch {
                result_id: "2".to_owned(),
            },
        ]
    );
}

#[test]
fn an_eval_run_past_the_age_limit_is_abandoned_with_a_reason() {
    let run = eval_run("running", MAX_EVAL_AGE + Duration::minutes(1));
    let plan = plan_resume(&run, &[result("1", "running", Some("run-1"))], now());
    assert!(matches!(
        plan.as_slice(),
        [ResumeAction::Abandon { result_id, reason }]
            if result_id == "1" && reason.contains("restarted")
    ));
}

#[test]
fn an_eval_run_just_inside_the_age_limit_is_resumed() {
    let run = eval_run("running", MAX_EVAL_AGE - Duration::minutes(1));
    let plan = plan_resume(&run, &[result("1", "running", Some("run-1"))], now());
    assert!(matches!(plan.as_slice(), [ResumeAction::Settle { .. }]));
}
