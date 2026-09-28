//! One colour vocabulary for every figure on the Analysis pages, so the same
//! number is the same colour wherever it appears. Tones are the console's
//! four badge tones: `ok`, `warn`, `err`, `muted`.

#[must_use]
pub(crate) const fn score_tone(score: i16) -> &'static str {
    if score >= 80 {
        "ok"
    } else if score >= 50 {
        "warn"
    } else {
        "err"
    }
}

#[must_use]
pub(crate) fn completion_tone(score: Option<f64>) -> &'static str {
    score.map_or("muted", |s| score_tone(s.round() as i16))
}

#[must_use]
pub(crate) const fn error_rate_tone(errors: i64, total: i64) -> &'static str {
    if total == 0 || errors == 0 {
        "ok"
    } else if errors * 10 >= total {
        "err"
    } else {
        "warn"
    }
}

#[must_use]
pub(crate) const fn deny_tone(denied: i64) -> &'static str {
    if denied == 0 {
        "muted"
    } else if denied >= 5 {
        "err"
    } else {
        "warn"
    }
}

const P95_HEALTHY_MS: f64 = 8_000.0;
const GATEWAY_P95_TARGET_MS: f64 = 20_000.0;

#[must_use]
pub(crate) fn latency_tone(p95_ms: Option<f64>) -> &'static str {
    match p95_ms {
        None => "muted",
        Some(ms) if ms <= P95_HEALTHY_MS => "ok",
        Some(ms) if ms <= GATEWAY_P95_TARGET_MS => "warn",
        Some(_) => "err",
    }
}

#[must_use]
pub(crate) const fn cache_tone(cache_tokens: i64, input_tokens: i64) -> &'static str {
    let total = cache_tokens + input_tokens;
    if total == 0 {
        "muted"
    } else if cache_tokens * 2 >= total {
        "ok"
    } else if cache_tokens * 5 >= total {
        "warn"
    } else {
        "err"
    }
}

#[must_use]
pub(crate) const fn safety_tone(findings: i64, blocked: i64) -> &'static str {
    if blocked > 0 {
        "err"
    } else if findings > 0 {
        "warn"
    } else {
        "muted"
    }
}

#[must_use]
pub(crate) fn percent(part: i64, whole: i64) -> String {
    if whole == 0 {
        "—".to_owned()
    } else {
        format!("{}%", (part * 100) / whole)
    }
}

#[must_use]
pub(crate) fn score_display(score: Option<f64>) -> String {
    score.map_or_else(|| "—".to_owned(), |s| format!("{s:.0}"))
}
