//! Evals on every trigger, against a real schema: a launch naming a case
//! records a one-case eval run with the caller's trigger, links the farm run
//! to the case's result, and the eval history folds finished eval runs per
//! agent per day while leaving running ones out. A scheduled launch naming a
//! case is the same eval, recorded as started by the scheduler. No executor
//! is started.

use serde_json::json;
use systemprompt_web_admin_farm::repositories::evals;
use systemprompt_web_admin_farm::repositories::filters::{NewEvalCase, NewEvalSuite};
use systemprompt_web_admin_farm::services::evals::{CaseLaunch, start_case_eval};
use systemprompt_web_admin_farm::services::farm_sync::eval_suite_id;
use systemprompt_web_admin_farm::services::runs::LaunchRequest;
use systemprompt_web_admin_farm::services::scheduled::{ScheduledLaunch, launch_scheduled};
use systemprompt_web_admin_farm::types::{EvalRunId, EvalRunStatus, TriggeredVia};

use crate::farm_runner_lifecycle::{Seeded, seed};
use crate::tempdb::TempDb;

async fn seed_suite(seeded: &Seeded) {
    let pool = &seeded.state.pool;
    let suite_id = eval_suite_id(&seeded.farm, seeded.agent.as_str());
    evals::upsert_eval_suite(
        pool,
        &NewEvalSuite {
            id: suite_id.clone(),
            farm_id: seeded.farm.clone(),
            agent_id: seeded.agent.clone(),
            source_path: "evals/reviewer.yaml".to_owned(),
            case_count: 1,
            synced_commit_sha: "abc123".to_owned(),
        },
    )
    .await
    .expect("upsert suite");
    evals::replace_eval_cases(
        pool,
        &suite_id,
        &[NewEvalCase {
            id: "flags-sql".to_owned(),
            prompt: Some("Flag the raw SQL.".to_owned()),
            fixture: None,
            timeout_secs: Some(300),
            assertions: json!([]),
            judge_rubric: None,
            judge_threshold: None,
        }],
    )
    .await
    .expect("replace cases");
}

fn request(seeded: &Seeded) -> LaunchRequest {
    LaunchRequest {
        farm_id: seeded.farm.clone(),
        agent_id: seeded.agent.clone(),
        user_id: seeded.user.clone(),
        triggered_via: TriggeredVia::Mcp,
        prompt: None,
        case_prompt: None,
        fixture: None,
        model: None,
        max_turns: None,
        max_budget_usd: None,
        timeout_seconds: None,
        eval_result_id: None,
        client_id: None,
    }
}

#[tokio::test]
async fn a_launch_with_a_case_is_an_eval_of_that_case() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let seeded = seed(&db.pool).await;
    seed_suite(&seeded).await;
    let launched = start_case_eval(
        std::sync::Arc::clone(&seeded.state),
        None,
        CaseLaunch {
            request: request(&seeded),
            case_id: "flags-sql".to_owned(),
        },
    )
    .await
    .expect("case eval");
    assert_eq!(launched.eval_run.triggered_via, "mcp");
    assert_eq!(launched.run.triggered_via, "mcp");
    assert_eq!(launched.run.timeout_seconds, 300);
    let results = evals::list_eval_results(&db.pool, &launched.eval_run.id)
        .await
        .expect("results");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].case_id, "flags-sql");
    assert_eq!(results[0].run_id.as_deref(), Some(launched.run.id.as_str()));
    assert_eq!(
        launched.run.eval_result_id.as_deref(),
        Some(results[0].id.as_str())
    );
    let eval_run = evals::find_eval_run(&db.pool, &launched.eval_run.id)
        .await
        .expect("find")
        .expect("eval run");
    assert_eq!(eval_run.status, EvalRunStatus::Running.as_str());
    db.cleanup().await;
}

#[tokio::test]
async fn an_unknown_case_records_nothing() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let seeded = seed(&db.pool).await;
    seed_suite(&seeded).await;
    let refused = start_case_eval(
        std::sync::Arc::clone(&seeded.state),
        None,
        CaseLaunch {
            request: request(&seeded),
            case_id: "no-such-case".to_owned(),
        },
    )
    .await;
    assert!(refused.is_err());
    let suite_id = eval_suite_id(&seeded.farm, seeded.agent.as_str());
    let runs = evals::list_recent_eval_runs(&db.pool, None, Some(&suite_id), 10)
        .await
        .expect("runs");
    assert!(runs.is_empty());
    db.cleanup().await;
}

async fn finished_eval_run(seeded: &Seeded, passed: i32, failed: i32, score: Option<f64>) {
    let pool = &seeded.state.pool;
    let id = EvalRunId::new(uuid::Uuid::new_v4().to_string());
    evals::insert_eval_run(
        pool,
        &evals::NewEvalRun {
            id: &id,
            suite_id: &eval_suite_id(&seeded.farm, seeded.agent.as_str()),
            farm_id: &seeded.farm,
            agent_id: &seeded.agent,
            user_id: &seeded.user,
            triggered_via: TriggeredVia::Api,
            client_id: None,
        },
    )
    .await
    .expect("insert eval run");
    evals::update_eval_run_counts(pool, &id, passed, failed, 0)
        .await
        .expect("counts");
    evals::update_eval_run_finished(pool, &id, EvalRunStatus::Completed, score, 400_000)
        .await
        .expect("finish");
}

#[tokio::test]
async fn history_folds_finished_eval_runs_per_agent_per_day() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let seeded = seed(&db.pool).await;
    seed_suite(&seeded).await;
    finished_eval_run(&seeded, 1, 1, Some(60.0)).await;
    finished_eval_run(&seeded, 2, 0, Some(90.0)).await;
    start_case_eval(
        std::sync::Arc::clone(&seeded.state),
        None,
        CaseLaunch {
            request: request(&seeded),
            case_id: "flags-sql".to_owned(),
        },
    )
    .await
    .expect("running eval");
    let since = chrono::Utc::now() - chrono::Duration::days(1);
    let days = evals::list_eval_history(&db.pool, Some(&seeded.farm), Some(&seeded.agent), since)
        .await
        .expect("history");
    assert_eq!(days.len(), 1, "one agent, one day: {days:?}");
    let today = &days[0];
    assert_eq!(today.runs, 2, "the running eval is left out");
    assert_eq!(today.passed, 3);
    assert_eq!(today.decided, 4);
    assert_eq!(today.mean_score, Some(75.0));
    assert_eq!(today.cost_microdollars, 800_000);
    db.cleanup().await;
}

#[tokio::test]
async fn a_scheduled_launch_with_a_case_is_an_eval_the_scheduler_started() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let seeded = seed(&db.pool).await;
    seed_suite(&seeded).await;
    let scheduled = |launcher: &str, case: Option<&str>| ScheduledLaunch {
        farm_id: seeded.farm.clone(),
        agent_id: seeded.agent.clone(),
        launcher: launcher.to_owned(),
        case: case.map(str::to_owned),
    };
    let run = launch_scheduled(
        &seeded.state,
        scheduled(seeded.user.as_str(), Some("flags-sql")),
    )
    .await
    .expect("scheduled launch")
    .expect("the launcher exists");
    assert_eq!(run.triggered_via, "scheduler");
    assert_eq!(run.user_id, seeded.user);
    let result_id = run
        .eval_result_id
        .clone()
        .expect("the run is an eval of its case");
    let suite_id = eval_suite_id(&seeded.farm, seeded.agent.as_str());
    let eval_runs = evals::list_recent_eval_runs(&db.pool, None, Some(&suite_id), 10)
        .await
        .expect("eval runs");
    assert_eq!(eval_runs.len(), 1);
    assert_eq!(eval_runs[0].triggered_via, "scheduler");
    let results = evals::list_eval_results(&db.pool, &eval_runs[0].id)
        .await
        .expect("results");
    assert_eq!(results[0].id.as_str(), result_id);
    assert_eq!(results[0].case_id, "flags-sql");

    let absent = launch_scheduled(
        &seeded.state,
        scheduled("nobody@test.invalid", Some("flags-sql")),
    )
    .await
    .expect("an absent launcher is not an error");
    assert!(absent.is_none(), "nothing is queued without the launcher");
    db.cleanup().await;
}

#[tokio::test]
async fn a_cancel_from_any_replica_stops_the_driver_and_survives_its_close() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let seeded = seed(&db.pool).await;
    seed_suite(&seeded).await;
    let pool = &seeded.state.pool;
    let id = EvalRunId::new(uuid::Uuid::new_v4().to_string());
    evals::insert_eval_run(
        pool,
        &evals::NewEvalRun {
            id: &id,
            suite_id: &eval_suite_id(&seeded.farm, seeded.agent.as_str()),
            farm_id: &seeded.farm,
            agent_id: &seeded.agent,
            user_id: &seeded.user,
            triggered_via: TriggeredVia::Api,
            client_id: None,
        },
    )
    .await
    .expect("insert eval run");
    evals::update_eval_run_status(pool, &id, EvalRunStatus::Running)
        .await
        .expect("running");
    assert!(!evals::is_eval_run_cancelled(pool, &id).await.expect("read"));

    systemprompt_web_admin_farm::services::evals::cancel_eval_run(&seeded.state, &id)
        .await
        .expect("cancel");

    assert!(
        evals::is_eval_run_cancelled(pool, &id).await.expect("read"),
        "the driver sees the cancel through the database, not process memory"
    );
    evals::update_eval_run_status(pool, &id, EvalRunStatus::Running)
        .await
        .expect("late running");
    evals::update_eval_run_finished(pool, &id, EvalRunStatus::Completed, Some(80.0), 1_000)
        .await
        .expect("close");
    let row = evals::find_eval_run(pool, &id)
        .await
        .expect("find")
        .expect("eval run");
    assert_eq!(row.status, EvalRunStatus::Cancelled.as_str());
    assert!(
        row.finished_at.is_some(),
        "closing still records the tallies"
    );
    assert_eq!(row.cost_microdollars, 1_000);
    db.cleanup().await;
}
