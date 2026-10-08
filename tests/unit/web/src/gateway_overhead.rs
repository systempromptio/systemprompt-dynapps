//! The nightly gateway overhead rollup's pure rules: which UTC day a run
//! rolls up, where the 400-day retention cuts, and which tone a night's p95
//! earns on the overview. The percentiles themselves are Postgres
//! `percentile_cont` over `latency_ms - upstream_latency_ms`, so they are not
//! re-implemented here.

use chrono::NaiveDate;

use systemprompt_web_admin_analytics::repositories::overview::gateway_overhead::{
    ALL_DIALECTS, ERR_P95_MS, KEEP_DAYS, WARN_P95_MS, overhead_tone,
};
use systemprompt_web_extension::jobs::internals::{previous_utc_day, retention_cutoff};

fn day(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
}

#[test]
fn rolls_up_the_closed_previous_day() {
    assert_eq!(previous_utc_day(day(2026, 10, 8)), day(2026, 10, 7));
}

#[test]
fn previous_day_crosses_month_and_year() {
    assert_eq!(previous_utc_day(day(2026, 3, 1)), day(2026, 2, 28));
    assert_eq!(previous_utc_day(day(2027, 1, 1)), day(2026, 12, 31));
}

#[test]
fn retention_keeps_exactly_four_hundred_days() {
    assert_eq!(KEEP_DAYS, 400);
    let today = day(2026, 10, 8);
    let cutoff = retention_cutoff(today);
    assert_eq!((today - cutoff).num_days(), 400);
    assert!(cutoff < previous_utc_day(today));
}

#[test]
fn tone_is_ok_below_the_warn_threshold() {
    assert_eq!(overhead_tone(0.0), "ok");
    assert_eq!(overhead_tone(WARN_P95_MS - 0.1), "ok");
}

#[test]
fn tone_warns_from_the_warn_threshold() {
    assert_eq!(overhead_tone(WARN_P95_MS), "warn");
    assert_eq!(overhead_tone(ERR_P95_MS - 0.1), "warn");
}

#[test]
fn tone_errs_from_the_err_threshold() {
    assert_eq!(overhead_tone(ERR_P95_MS), "err");
    assert_eq!(overhead_tone(60_000.0), "err");
}

#[test]
fn all_dialects_key_is_stable() {
    assert_eq!(ALL_DIALECTS, "all");
}
