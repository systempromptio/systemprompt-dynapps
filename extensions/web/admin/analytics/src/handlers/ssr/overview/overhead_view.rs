//! The gateway overhead block: last night's p50/p95/p99 of the time the
//! gateway added around the provider call, each with a 30-day sparkline, and
//! the 30 nightly rows behind them (NFR-1.6).
//!
//! It reads `gateway_overhead_daily`, which the nightly
//! `gateway_overhead_rollup` job writes, so it does not move with the window
//! control: a closed day's percentiles never change once rolled up.

use serde::Serialize;

use crate::handlers::ssr::ssr_analytics_dashboard::ms;
use crate::repositories::overview::gateway_overhead::{GatewayOverheadDay, overhead_tone};
use systemprompt_web_admin_core::ssr::format::short_num;
use systemprompt_web_admin_core::ssr::types::sparkline::{SparklineView, sparkline_toned};

pub(super) const OVERHEAD_DAYS: i64 = 30;

#[derive(Debug, Serialize)]
pub(super) struct OverheadView {
    pub error: Option<String>,
    pub has_rows: bool,
    pub latest_date: String,
    pub tiles: Vec<OverheadTileView>,
    pub rows: Vec<OverheadRowView>,
}

#[derive(Debug, Serialize)]
pub(super) struct OverheadTileView {
    pub key: &'static str,
    pub label: &'static str,
    pub value: String,
    pub note: String,
    pub tone: &'static str,
    pub spark: SparklineView,
}

#[derive(Debug, Serialize)]
pub(super) struct OverheadRowView {
    pub date: String,
    pub requests_display: String,
    pub p50_display: String,
    pub p95_display: String,
    pub p99_display: String,
    pub upstream_p95_display: String,
}

pub(super) fn overhead_view(loaded: &Result<Vec<GatewayOverheadDay>, sqlx::Error>) -> OverheadView {
    let days = loaded.as_deref().unwrap_or(&[]);
    let error = loaded
        .as_ref()
        .err()
        .map(|e| format!("Gateway overhead could not be read: {e}"));
    let Some(latest) = days.first() else {
        return OverheadView {
            error,
            has_rows: false,
            latest_date: String::new(),
            tiles: Vec::new(),
            rows: Vec::new(),
        };
    };
    let tone = overhead_tone(latest.overhead_p95_ms);
    let specs = [
        TileSpec {
            key: "overhead-p50",
            label: "Overhead p50",
            pick: |d| d.overhead_p50_ms,
            note: format!("upstream {}", ms(Some(latest.upstream_p50_ms))),
        },
        TileSpec {
            key: "overhead-p95",
            label: "Overhead p95",
            pick: |d| d.overhead_p95_ms,
            note: format!("upstream {}", ms(Some(latest.upstream_p95_ms))),
        },
        TileSpec {
            key: "overhead-p99",
            label: "Overhead p99",
            pick: |d| d.overhead_p99_ms,
            note: format!("{} requests", short_num(latest.request_count)),
        },
    ];
    let tiles = specs
        .into_iter()
        .map(|spec| tile(spec, tone, days))
        .collect();
    OverheadView {
        error,
        has_rows: true,
        latest_date: latest.date.format("%Y-%m-%d").to_string(),
        tiles,
        rows: days.iter().map(row).collect(),
    }
}

struct TileSpec {
    key: &'static str,
    label: &'static str,
    pick: fn(&GatewayOverheadDay) -> f64,
    note: String,
}

fn tile(spec: TileSpec, tone: &'static str, days: &[GatewayOverheadDay]) -> OverheadTileView {
    let series: Vec<i64> = days
        .iter()
        .rev()
        .map(|d| (spec.pick)(d).round() as i64)
        .collect();
    let latest = days.first().map(spec.pick);
    OverheadTileView {
        key: spec.key,
        label: spec.label,
        value: ms(latest),
        note: spec.note,
        tone,
        spark: sparkline_toned(
            &series,
            tone,
            format!("{} per night, last 30 days", spec.label),
        ),
    }
}

fn row(day: &GatewayOverheadDay) -> OverheadRowView {
    OverheadRowView {
        date: day.date.format("%Y-%m-%d").to_string(),
        requests_display: short_num(day.request_count),
        p50_display: ms(Some(day.overhead_p50_ms)),
        p95_display: ms(Some(day.overhead_p95_ms)),
        p99_display: ms(Some(day.overhead_p99_ms)),
        upstream_p95_display: ms(Some(day.upstream_p95_ms)),
    }
}
