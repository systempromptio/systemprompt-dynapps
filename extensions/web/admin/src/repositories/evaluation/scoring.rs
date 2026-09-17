//! Deterministic scores for the speed and cost criteria.
//!
//! Both are a ratio of target to actual, clamped to 0–100, so hitting the
//! target scores 100 and taking twice the budget scores 50. The judge never
//! chooses these numbers; it is told them and asked to explain.

use super::metrics::SessionMetrics;

const ERROR_PENALTY: f64 = 5.0;
const CACHE_BONUS: f64 = 10.0;

#[must_use]
pub fn speed_score(
    m: &SessionMetrics,
    target_minutes_per_prompt: f64,
    target_latency_ms: i64,
) -> u8 {
    let prompts = m.user_prompts.max(1) as f64;
    let minutes_per_prompt = m.duration_minutes / prompts;
    let pace = ratio(target_minutes_per_prompt, minutes_per_prompt);
    let latency = m
        .avg_latency_ms
        .map_or(1.0, |actual| ratio(target_latency_ms as f64, actual));
    let blended = 0.7f64.mul_add(pace, 0.3 * latency) * 100.0;
    let penalised = ERROR_PENALTY.mul_add(-(m.errors as f64), blended);
    clamp(penalised)
}

#[must_use]
pub fn cost_score(
    m: &SessionMetrics,
    budget_microdollars_per_prompt: i64,
    expected_cache_read_ratio: f64,
) -> u8 {
    let prompts = m.user_prompts.max(1) as f64;
    let cost_per_prompt = m.cost_microdollars as f64 / prompts;
    let mut score = ratio(budget_microdollars_per_prompt as f64, cost_per_prompt) * 100.0;
    let context = m.input_tokens + m.cache_read_tokens + m.cache_creation_tokens;
    if context > 0 {
        let cache_ratio = m.cache_read_tokens as f64 / context as f64;
        if cache_ratio >= expected_cache_read_ratio {
            score += CACHE_BONUS;
        }
    }
    clamp(score)
}

// Why: a session that beats the target is not "better than perfect" — the
// ratio caps at 1 so a trivially short session cannot mask a slow one in the
// average.
fn ratio(target: f64, actual: f64) -> f64 {
    if actual <= 0.0 {
        return 1.0;
    }
    (target / actual).clamp(0.0, 1.0)
}

const fn clamp(score: f64) -> u8 {
    score.round().clamp(0.0, 100.0) as u8
}
