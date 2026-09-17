//! The deterministic speed and cost scorers: a session on target scores 100,
//! one at twice the target scores half, and nothing can leave 0–100.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: panics are the assertion mechanism"
)]

use systemprompt_web_admin::repositories::evaluation::metrics::SessionMetrics;
use systemprompt_web_admin::repositories::evaluation::scoring::{cost_score, speed_score};

fn session(prompts: i64, minutes: f64, latency_ms: f64, cost: i64) -> SessionMetrics {
    SessionMetrics {
        requests: prompts,
        completed: prompts,
        user_prompts: prompts,
        duration_minutes: minutes,
        avg_latency_ms: Some(latency_ms),
        cost_microdollars: cost,
        input_tokens: 1_000,
        ..SessionMetrics::default()
    }
}

#[test]
fn speed_on_target_scores_full_marks() {
    let m = session(4, 16.0, 20_000.0, 0);
    assert_eq!(speed_score(&m, 4.0, 20_000), 100);
}

#[test]
fn speed_faster_than_target_does_not_exceed_full_marks() {
    let m = session(4, 2.0, 1_000.0, 0);
    assert_eq!(speed_score(&m, 4.0, 20_000), 100);
}

#[test]
fn speed_at_twice_the_target_scores_half() {
    let m = session(4, 32.0, 40_000.0, 0);
    assert_eq!(speed_score(&m, 4.0, 20_000), 50);
}

#[test]
fn speed_errors_cost_five_points_each_and_floor_at_zero() {
    let mut m = session(4, 16.0, 20_000.0, 0);
    m.errors = 2;
    assert_eq!(speed_score(&m, 4.0, 20_000), 90);
    m.errors = 40;
    assert_eq!(speed_score(&m, 4.0, 20_000), 0);
}

#[test]
fn cost_on_budget_scores_full_marks() {
    let m = session(4, 1.0, 1.0, 1_000_000);
    assert_eq!(cost_score(&m, 250_000, 0.5), 100);
}

#[test]
fn cost_at_twice_the_budget_scores_half() {
    let m = session(4, 1.0, 1.0, 2_000_000);
    assert_eq!(cost_score(&m, 250_000, 0.5), 50);
}

#[test]
fn cache_reuse_adds_a_bonus_capped_at_full_marks() {
    let mut m = session(4, 1.0, 1.0, 2_000_000);
    m.cache_read_tokens = 3_000;
    assert_eq!(cost_score(&m, 250_000, 0.5), 60);
    m.cost_microdollars = 1_000_000;
    assert_eq!(cost_score(&m, 250_000, 0.5), 100);
}

#[test]
fn zero_prompts_are_treated_as_one() {
    let m = session(0, 4.0, 20_000.0, 250_000);
    assert_eq!(speed_score(&m, 4.0, 20_000), 100);
    assert_eq!(cost_score(&m, 250_000, 0.5), 100);
}
