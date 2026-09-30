//! Spend-tab view builder: the fast/slow latency split at the caller's SLO.
//!
//! Split from `view.rs` at the 300-line ceiling; the shared label helpers
//! stay there and are imported here.

use crate::repositories::analytics::site::latency::LatencySplit;
use systemprompt_web_admin_core::ssr::format::format_duration_ms;

use super::context::FastSlowView;

pub(super) fn fast_slow(split: &LatencySplit) -> FastSlowView {
    let timed = split.fast + split.slow;
    let threshold = format_duration_ms(i64::from(split.threshold_ms));
    FastSlowView {
        within_label: format!("Within SLO (<{threshold})"),
        breach_label: format!("Breaching SLO (>={threshold})"),
        fast: split.fast,
        slow: split.slow,
        untimed: split.untimed,
        threshold_display: threshold,
        breach_pct_display: if timed > 0 {
            let permille = split.slow.saturating_mul(1000) / timed;
            format!("{}.{}%", permille / 10, permille % 10)
        } else {
            "–".to_owned()
        },
        p50_display: format_duration_ms(split.p50_ms.round() as i64),
        p95_display: format_duration_ms(split.p95_ms.round() as i64),
        has_data: timed + split.untimed > 0,
    }
}
