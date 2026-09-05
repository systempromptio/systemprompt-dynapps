//! The KPI strip: six figures, each with its window-over-window delta and a
//! link to the page that owns the rows it counted.
//!
//! The Requests tile carries the sparkline, so the strip answers "how much,
//! and was it steady" in one glance rather than reserving a second card for
//! the shape.

use serde::Serialize;

use crate::handlers::ssr::format::{format_cost, format_duration_ms, short_num};
use crate::handlers::ssr::types::{SparklineView, sparkline};
use crate::repositories::overview::series::BUCKETS;
use crate::util::delta::{Delta, delta};

use super::data::OverviewData;
use super::view::OverviewRange;

#[derive(Debug, Serialize)]
pub(super) struct KpiStripView {
    pub error: Option<String>,
    pub requests: KpiTileView,
    pub spark: SparklineView,
    pub spark_caption: String,
    pub tiles: Vec<KpiTileView>,
}

#[derive(Debug, Serialize)]
pub(super) struct KpiTileView {
    pub label: &'static str,
    pub value: String,
    pub sub: String,
    pub href: String,
    pub tone: &'static str,
    pub delta: String,
    pub delta_dir: &'static str,
}

pub(super) fn kpi_strip(range: OverviewRange, data: &OverviewData) -> KpiStripView {
    let k = data.kpis.as_ref().copied().unwrap_or_default();
    let preset = range.preset();
    let vs = range.previous_label();
    KpiStripView {
        error: data
            .kpis
            .as_ref()
            .err()
            .map(|e| format!("The headline figures could not be read: {e}")),
        requests: tile(
            "Requests",
            short_num(k.requests),
            format!("{} {vs}", short_num(k.prev_requests)),
            format!("/admin/requests?preset={preset}"),
            delta(k.requests, k.prev_requests, true),
            "",
        ),
        spark: data
            .buckets
            .as_deref()
            .map_or_else(|_| sparkline(&[]), sparkline),
        spark_caption: spark_caption(range),
        tiles: vec![
            tile(
                "Spend",
                format_cost(k.cost_microdollars),
                format!("{} {vs}", format_cost(k.prev_cost_microdollars)),
                format!("/admin/analytics?tab=cost&preset={preset}"),
                delta(k.cost_microdollars, k.prev_cost_microdollars, false),
                "",
            ),
            tile(
                "Active people",
                k.active_users.to_string(),
                format!("{} {vs}", k.prev_active_users),
                "/admin/users".to_owned(),
                delta(k.active_users, k.prev_active_users, true),
                "",
            ),
            tile(
                "p95 latency",
                format_duration_ms(k.p95_latency_ms),
                format!("{} {vs}", format_duration_ms(k.prev_p95_latency_ms)),
                format!("/admin/traces?preset={preset}"),
                delta(k.p95_latency_ms, k.prev_p95_latency_ms, false),
                "",
            ),
            tile(
                "Error rate",
                percent_tenths(k.error_rate_tenths(), k.requests),
                format!("{} failed of {}", k.errors, short_num(k.requests)),
                format!("/admin/requests?status=failed&preset={preset}"),
                delta(k.error_rate_tenths(), k.prev_error_rate_tenths(), false),
                if k.errors > 0 { "err" } else { "" },
            ),
            tile(
                "Denied",
                k.denied.to_string(),
                format!("{} {vs}", k.prev_denied),
                format!("/admin/requests?status=rejected&preset={preset}"),
                delta(k.denied, k.prev_denied, false),
                if k.denied > 0 { "warn" } else { "" },
            ),
        ],
    }
}

fn tile(
    label: &'static str,
    value: String,
    sub: String,
    href: String,
    d: Delta,
    tone: &'static str,
) -> KpiTileView {
    KpiTileView {
        label,
        value,
        sub,
        href,
        tone,
        delta: d.display(),
        delta_dir: delta_dir(d),
    }
}

// Why: the delta component takes one direction class, not a Delta — polarity
// and direction fold into that token here so the template branches on nothing.
// A flat delta gets no modifier and keeps the base arrow.
fn delta_dir(d: Delta) -> &'static str {
    match (d.direction, d.tone) {
        ("up", "bad") => "up-bad",
        ("down", "good") => "down-good",
        ("up", _) => "up",
        ("down", _) => "down",
        _ => "",
    }
}

// Why: an empty window has no rate to report; a zero would claim a
// measurement nobody took.
fn percent_tenths(tenths: i64, sample: i64) -> String {
    if sample <= 0 {
        return "\u{2014}".to_owned();
    }
    format!("{}.{}%", tenths / 10, tenths % 10)
}

// Why: the sparkline always draws the same number of points, so what one point
// covers changes with the window and the caption has to say which it is.
fn spark_caption(range: OverviewRange) -> String {
    match range.hours() / BUCKETS {
        1 => "requests per hour".to_owned(),
        n => format!("requests per {n}h"),
    }
}
