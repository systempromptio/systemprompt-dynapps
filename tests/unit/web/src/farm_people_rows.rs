//! The by-person leaderboard rows: each person's share of the window's runs,
//! the dashboard / API / eval split, and the success label.

use chrono::Utc;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_farm::repositories::rows::RunsByUserRow;
use systemprompt_web_admin_farm::views::people::{
    person_rows, share_pct, success_label, total_runs, trigger_split,
};

fn row(user: &str, runs: i64, dashboard: i64, api: i64, eval: i64) -> RunsByUserRow {
    RunsByUserRow {
        user_id: UserId::new(user),
        display_name: Some(format!("{user} name")),
        runs: Some(runs),
        succeeded: Some(runs - 1),
        failed: Some(1),
        active: Some(0),
        dashboard_runs: Some(dashboard),
        api_runs: Some(api),
        eval_runs: Some(eval),
        cost_microdollars: Some(1_500_000),
        favourite_agent: Some("reviewer".to_owned()),
        last_run_at: Some(Utc::now()),
    }
}

#[test]
fn share_is_a_rounded_percentage_of_the_total() {
    assert!((share_pct(1, 3) - 33.3).abs() < f64::EPSILON);
    assert!((share_pct(3, 3) - 100.0).abs() < f64::EPSILON);
    assert!((share_pct(5, 0)).abs() < f64::EPSILON);
    assert!((share_pct(0, 10)).abs() < f64::EPSILON);
    assert!((share_pct(20, 10) - 100.0).abs() < f64::EPSILON);
}

#[test]
fn shares_across_people_sum_to_the_window() {
    let rows = vec![
        row("u-a", 6, 6, 0, 0),
        row("u-b", 3, 0, 3, 0),
        row("u-c", 1, 0, 0, 1),
    ];
    assert_eq!(total_runs(&rows), 10);
    let views = person_rows(&rows);
    for (view, want) in views.iter().zip([60.0, 30.0, 10.0]) {
        assert!((view.share_pct - want).abs() < f64::EPSILON);
    }
    assert_eq!(views[0].share_label, "60%");
    assert_eq!(views[0].user_url, "/admin/users/u-a");
    assert_eq!(views[0].runs_url, "/admin/farm/runs?user_id=u-a");
    assert_eq!(views[0].display_name, "u-a name");
}

#[test]
fn trigger_split_fills_the_bar_exactly() {
    let split = trigger_split(1, 1, 1);
    let sum = split.dashboard + split.api + split.eval;
    assert!((sum - 100.0).abs() < 0.05, "sum was {sum}");
    assert!((split.dashboard - 33.3).abs() < f64::EPSILON);
    assert!((split.eval - 33.4).abs() < 0.05);
}

#[test]
fn trigger_split_of_a_single_source_is_whole() {
    let split = trigger_split(0, 4, 0);
    assert!((split.api - 100.0).abs() < f64::EPSILON);
    assert!(split.dashboard.abs() < f64::EPSILON);
    assert!(split.eval.abs() < f64::EPSILON);
    let empty = trigger_split(0, 0, 0);
    assert!((empty.dashboard + empty.api + empty.eval).abs() < f64::EPSILON);
}

#[test]
fn success_label_counts_finished_runs_only() {
    assert_eq!(success_label(9, 1), "90%");
    assert_eq!(success_label(0, 0), "—");
    assert_eq!(success_label(2, 1), "67%");
}
