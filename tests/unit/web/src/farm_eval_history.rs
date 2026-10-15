//! Eval history: the `since` forms the API and MCP accept, a day row's pass
//! rate and dollar cost, the per-agent trend rows the dashboard draws, the
//! recent eval runs table with its trigger badges, and the trigger values
//! every surface records.

use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_farm::repositories::rows::{EvalDayRow, EvalRunRow};
use systemprompt_web_admin_farm::services::evals::{EvalHistoryPoint, parse_since};
use systemprompt_web_admin_farm::types::{
    EvalRunId, EvalSuiteId, FarmAgentId, FarmId, TriggeredVia,
};
use systemprompt_web_admin_farm::views::eval_history::{eval_run_items, eval_trend_rows};
use systemprompt_web_admin_farm::views::status::trigger_badge;

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 14, 12, 0, 0)
        .single()
        .expect("time")
}

fn day(agent: &str, d: u32, passed: i64, decided: i64, score: Option<f64>) -> EvalDayRow {
    EvalDayRow {
        farm_id: FarmId::new("farm"),
        agent_id: FarmAgentId::new(agent),
        day: NaiveDate::from_ymd_opt(2026, 10, d).expect("date"),
        runs: 1,
        passed,
        decided,
        mean_score: score,
        cost_microdollars: 250_000,
    }
}

fn eval_run(via: &str, status: &str, score: Option<f64>) -> EvalRunRow {
    EvalRunRow {
        id: EvalRunId::new("er-1"),
        suite_id: EvalSuiteId::new("farm.reviewer"),
        farm_id: FarmId::new("farm"),
        agent_id: FarmAgentId::new("reviewer"),
        user_id: UserId::new("history-test-user"),
        status: status.to_owned(),
        started_at: None,
        finished_at: None,
        passed: 2,
        failed: 1,
        errored: 0,
        mean_score: score,
        cost_microdollars: 1_500_000,
        created_at: now(),
        triggered_via: via.to_owned(),
    }
}

#[test]
fn since_defaults_to_thirty_days() {
    assert_eq!(parse_since("", now()), Some(now() - Duration::days(30)));
    assert_eq!(parse_since("  ", now()), Some(now() - Duration::days(30)));
}

#[test]
fn since_reads_relative_windows() {
    assert_eq!(parse_since("7d", now()), Some(now() - Duration::days(7)));
    assert_eq!(parse_since("24h", now()), Some(now() - Duration::hours(24)));
}

#[test]
fn since_reads_an_rfc3339_instant() {
    let at = Utc
        .with_ymd_and_hms(2026, 10, 1, 0, 0, 0)
        .single()
        .expect("time");
    assert_eq!(parse_since("2026-10-01T00:00:00Z", now()), Some(at));
}

#[test]
fn since_is_capped_at_a_year() {
    assert_eq!(
        parse_since("5000d", now()),
        Some(now() - Duration::days(366))
    );
}

#[test]
fn since_refuses_anything_else() {
    for raw in ["yesterday", "0d", "-3d", "7w", "d"] {
        assert_eq!(parse_since(raw, now()), None, "{raw}");
    }
}

#[test]
fn a_history_point_carries_pass_rate_and_dollars() {
    let point = EvalHistoryPoint::from(day("reviewer", 3, 3, 4, Some(81.5)));
    assert_eq!(point.pass_rate, Some(75.0));
    assert_eq!(point.mean_score, Some(81.5));
    assert!((point.cost_usd - 0.25).abs() < f64::EPSILON);
    let undecided = EvalHistoryPoint::from(day("reviewer", 3, 0, 0, None));
    assert_eq!(undecided.pass_rate, None);
}

#[test]
fn trend_rows_fold_days_per_agent() {
    let rows = eval_trend_rows(&[
        day("writer", 2, 1, 2, Some(40.0)),
        day("reviewer", 5, 2, 2, Some(90.0)),
        day("reviewer", 3, 1, 2, Some(60.0)),
    ]);
    assert_eq!(rows.len(), 2);
    let reviewer = &rows[0];
    assert_eq!(reviewer.agent_id.as_str(), "reviewer");
    assert_eq!(reviewer.runs, 2);
    assert_eq!(reviewer.days, 2);
    assert_eq!(reviewer.pass_display, "75%");
    assert_eq!(reviewer.score_display, "75");
    assert_eq!(reviewer.score_tone, "warn");
    assert_eq!(reviewer.agent_url, "/admin/farm/farm/agents/reviewer");
    let writer = &rows[1];
    assert_eq!(writer.pass_display, "50%");
    assert_eq!(writer.score_tone, "err");
}

#[test]
fn an_unjudged_agent_shows_no_score() {
    let rows = eval_trend_rows(&[day("reviewer", 3, 1, 1, None)]);
    assert_eq!(rows[0].score_display, "—");
    assert_eq!(rows[0].score_tone, "muted");
    assert_eq!(rows[0].pass_tone, "ok");
}

#[test]
fn recent_eval_runs_name_their_trigger() {
    let items = eval_run_items(&[
        eval_run("mcp", "completed", Some(88.0)),
        eval_run("demo", "running", None),
    ]);
    assert_eq!(items[0].trigger.text, "MCP");
    assert_eq!(items[0].status.text, "Completed");
    assert_eq!(items[0].score_display, "88");
    assert_eq!(items[0].href, "/admin/farm/evals/runs/er-1");
    assert_eq!(items[1].trigger.text, "Demo");
    assert_eq!(items[1].score_display, "—");
}

#[test]
fn every_trigger_round_trips_and_has_a_badge() {
    for via in TriggeredVia::ALL {
        assert_eq!(TriggeredVia::parse(via.as_str()), Some(via));
        assert_ne!(trigger_badge(via.as_str()).text, "Unknown", "{via}");
    }
    assert_eq!(trigger_badge("console").text, "Console");
    assert_eq!(trigger_badge("mcp").text, "MCP");
    assert_eq!(trigger_badge("demo").text, "Demo");
}
