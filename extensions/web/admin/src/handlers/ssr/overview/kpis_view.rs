//! The KPI strip: six figures, each with its window-over-window delta and a
//! link to the page that owns the rows it counted.
//!
//! The Requests tile carries the sparkline, so the strip answers "how much,
//! and was it steady" in one glance rather than reserving a second card for
//! the shape.

use serde::Serialize;

use crate::handlers::ssr::format::{format_cost, format_duration_ms, short_num};
use crate::handlers::ssr::types::{SparklineView, sparkline};
use crate::repositories::overview::kpis::OverviewKpis;
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
            Tile {
                label: "Requests",
                value: short_num(k.requests),
                sub: format!("{} {vs}", short_num(k.prev_requests)),
                href: format!("/admin/requests?preset={preset}"),
                tone: "",
            },
            delta(k.requests, k.prev_requests, true),
        ),
        spark: data
            .buckets
            .as_deref()
            .map_or_else(|_| sparkline(&[]), sparkline),
        spark_caption: spark_caption(range),
        tiles: secondary_tiles(&k, preset, vs),
    }
}

// Why: the five tiles after Requests, extracted so the strip builder stays
// readable — the sparkline tile is built inline because it is the only one
// that carries a chart.
fn secondary_tiles(k: &OverviewKpis, preset: &str, vs: &str) -> Vec<KpiTileView> {
    vec![
        tile(
            Tile {
                label: "Spend",
                value: format_cost(k.cost_microdollars),
                sub: format!("{} {vs}", format_cost(k.prev_cost_microdollars)),
                href: format!("/admin/analytics?tab=cost&preset={preset}"),
                tone: "",
            },
            delta(k.cost_microdollars, k.prev_cost_microdollars, false),
        ),
        tile(
            Tile {
                label: "Active people",
                value: k.active_users.to_string(),
                sub: format!("{} {vs}", k.prev_active_users),
                href: "/admin/users".to_owned(),
                tone: "",
            },
            delta(k.active_users, k.prev_active_users, true),
        ),
        tile(
            Tile {
                label: "p95 latency",
                value: format_duration_ms(k.p95_latency_ms),
                sub: format!("{} {vs}", format_duration_ms(k.prev_p95_latency_ms)),
                href: format!("/admin/traces?preset={preset}"),
                tone: "",
            },
            delta(k.p95_latency_ms, k.prev_p95_latency_ms, false),
        ),
        tile(
            Tile {
                label: "Error rate",
                value: percent_tenths(k.error_rate_tenths(), k.requests),
                sub: format!("{} failed of {}", k.errors, short_num(k.requests)),
                href: format!("/admin/requests?status=failed&preset={preset}"),
                tone: if k.errors > 0 { "err" } else { "" },
            },
            delta(k.error_rate_tenths(), k.prev_error_rate_tenths(), false),
        ),
        tile(
            Tile {
                label: "Denied",
                value: k.denied.to_string(),
                sub: format!("{} {vs}", k.prev_denied),
                href: format!("/admin/requests?status=rejected&preset={preset}"),
                tone: if k.denied > 0 { "warn" } else { "" },
            },
            delta(k.denied, k.prev_denied, false),
        ),
    ]
}

// Why: the tile's own fields travel together as one value so the builder takes
// two arguments rather than six — the delta is the only thing computed
// separately, because its polarity is the caller's call and not the tile's.
struct Tile {
    label: &'static str,
    value: String,
    sub: String,
    href: String,
    tone: &'static str,
}

fn tile(spec: Tile, d: Delta) -> KpiTileView {
    KpiTileView {
        label: spec.label,
        value: spec.value,
        sub: spec.sub,
        href: spec.href,
        tone: spec.tone,
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
