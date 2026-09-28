//! Pure value formatting shared by the web crates' display layers.
//!
//! One implementation per concern so a cost or duration renders identically
//! on every page that shows it. A cost of zero renders as `—`, not `$0`: no
//! billed traffic is not the same as traffic that cost nothing. A month is the
//! roster's thirty-day idle threshold, so "1mo ago" and the idle chip agree.

pub fn short_id(id: &str) -> String {
    const KEEP: usize = 12;
    if id.chars().count() > KEEP {
        let head: String = id.chars().take(KEEP).collect();
        format!("{head}…")
    } else {
        id.to_owned()
    }
}

const NO_BILLED_COST: &str = "—";

pub fn format_cost(microdollars: i64) -> String {
    if microdollars <= 0 {
        return NO_BILLED_COST.to_owned();
    }
    let dollars = microdollars as f64 / 1_000_000.0;
    if dollars >= 1.0 {
        format!("${dollars:.2}")
    } else if dollars >= 0.01 {
        format!("${dollars:.4}")
    } else {
        format!("${dollars:.6}")
    }
}
pub fn short_num(n: i64) -> String {
    let abs = n.unsigned_abs();
    if abs >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if abs >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
pub fn format_duration_ms(ms: i64) -> String {
    if ms < 1000 {
        format!("{ms} ms")
    } else if ms < 60_000 {
        format!("{:.2} s", ms as f64 / 1000.0)
    } else if ms < 3_600_000 {
        format!("{:.1} min", ms as f64 / 60_000.0)
    } else {
        format!("{:.1} h", ms as f64 / 3_600_000.0)
    }
}

pub fn truncate_chars(s: &str, max_chars: usize) -> &str {
    s.char_indices()
        .nth(max_chars)
        .map_or(s, |(end, _)| &s[..end])
}

pub fn truncate_ellipsis(s: &str, max_chars: usize) -> String {
    let head = truncate_chars(s, max_chars);
    if head.len() == s.len() {
        s.to_owned()
    } else {
        format!("{head}\u{2026}")
    }
}

pub fn compact_num(v: i64) -> String {
    if v >= 1_000_000 {
        format!("{:.1}M", v as f64 / 1_000_000.0)
    } else if v >= 10_000 {
        format!("{}k", v / 1000)
    } else if v >= 1000 {
        format!("{:.1}k", v as f64 / 1000.0)
    } else {
        v.to_string()
    }
}

const IDLE_MONTH_SECS: i64 = 30 * 86_400;

pub fn relative_time(delta_secs: i64) -> String {
    match delta_secs.max(0) {
        d if d < 60 => "just now".to_owned(),
        d if d < 3_600 => format!("{}m ago", d / 60),
        d if d < 86_400 => format!("{}h ago", d / 3_600),
        d if d < IDLE_MONTH_SECS => format!("{}d ago", d / 86_400),
        d => format!("{}mo ago", d / IDLE_MONTH_SECS),
    }
}
