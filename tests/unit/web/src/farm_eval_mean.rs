//! The pooled eval mean behind the farm KPI tiles and `farm_dashboard`'s
//! totals: each day's mean is weighted by how many runs it scored, so one
//! scored run on a quiet day does not count as much as a busy day's fifty,
//! and days with nothing scored (only infrastructure failures, say) are
//! left out rather than averaged in as zero.

use chrono::NaiveDate;
use systemprompt_web_admin_farm::repositories::clients::{DailyOutcomeRow, pooled_eval_mean};

fn day(n: u32, eval_mean: Option<f64>, eval_scored: i64, infra_failures: i64) -> DailyOutcomeRow {
    DailyOutcomeRow {
        day: NaiveDate::from_ymd_opt(2026, 10, n).expect("valid day"),
        runs: eval_scored + infra_failures,
        succeeded: eval_scored,
        finished: eval_scored + infra_failures,
        cost_microdollars: 0,
        eval_mean,
        eval_scored,
        infra_failures,
    }
}

#[test]
fn no_scored_run_is_no_mean() {
    assert_eq!(pooled_eval_mean(&[]), None);
    assert_eq!(pooled_eval_mean(&[day(1, None, 0, 3)]), None);
}

#[test]
fn a_day_of_only_infra_failures_does_not_drag_the_mean() {
    let days = [day(14, None, 0, 2), day(15, Some(90.0), 1, 1)];
    assert_eq!(pooled_eval_mean(&days), Some(90.0));
}

#[test]
fn days_are_weighted_by_runs_scored() {
    let days = [day(14, Some(100.0), 3, 0), day(15, Some(0.0), 1, 0)];
    assert_eq!(pooled_eval_mean(&days), Some(75.0));
}
