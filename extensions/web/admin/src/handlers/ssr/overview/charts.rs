//! The six charts under the KPI strip, all on one spine of buckets so a
//! spike in one lines up with the others: volume (requests, conversations,
//! people), spend by model, requests by client, tokens against cache reads,
//! tool calls against errors and denials, and latency.
//!
//! Every chart is the same server-rendered component the analysis pages use,
//! so the shape of the day reads the same here as it does one click deeper.

use chrono::{DateTime, Utc};

use crate::handlers::ssr::format::{client_label, format_cost, format_token_total, short_num};
use crate::handlers::ssr::ssr_analytics_dashboard::short_name;
use crate::handlers::ssr::types::{Plot, SvgLineChartView, SvgSeriesInput, chart_on_axis};
use crate::repositories::overview::dimension_series::{DimensionBucket, OverviewDimension};
use crate::repositories::overview::series::{BUCKETS, OverviewSeriesPoint};

use super::data::OverviewData;
use super::view::OverviewRange;

const EMPTY: &str = "No gateway traffic in this window.";

// Why: the shared x axis: one label per bucket, built once from the window's
// edges and handed to every chart, so the hover crosshair says the same time
// on all six.
struct Axis {
    labels: Vec<String>,
}

impl Axis {
    fn new(range: OverviewRange, n: usize) -> Self {
        let window = range.time_range();
        let width = (window.to - window.from) / i32::try_from(BUCKETS).unwrap_or(24);
        Self {
            labels: (0..n)
                .map(|i| bucket_label(window.from + width * i32::try_from(i).unwrap_or(0), range))
                .collect(),
        }
    }

    fn chart(&self, plot: Plot) -> SvgLineChartView {
        chart_on_axis(&self.labels, EMPTY, plot)
    }
}

// Why: a bucket is an hour at 24h, seven hours at 7d and thirty at 30d, so the
// label carries the day once the buckets stop fitting inside one.
fn bucket_label(t: DateTime<Utc>, range: OverviewRange) -> String {
    let local = t.with_timezone(&chrono::Local);
    match range {
        OverviewRange::Day => local.format("%H:%M").to_string(),
        OverviewRange::Week | OverviewRange::Month => local.format("%b %-d %H:%M").to_string(),
    }
}

fn input(
    series: &[OverviewSeriesPoint],
    label: &str,
    f: fn(&OverviewSeriesPoint) -> i64,
    total_display: String,
) -> SvgSeriesInput {
    SvgSeriesInput {
        label: label.to_owned(),
        values: series.iter().map(f).collect(),
        value_display: total_display,
    }
}

fn sum(series: &[OverviewSeriesPoint], f: fn(&OverviewSeriesPoint) -> i64) -> i64 {
    series.iter().map(f).sum()
}

fn peak(series: &[OverviewSeriesPoint], f: fn(&OverviewSeriesPoint) -> i64) -> i64 {
    series.iter().map(f).max().unwrap_or(0)
}

pub(super) fn charts(range: OverviewRange, data: &OverviewData) -> Vec<SvgLineChartView> {
    let series = data.series.as_deref().unwrap_or(&[]);
    let axis = Axis::new(range, series.len().max(1));
    vec![
        volume(&axis, series),
        dimension(&axis, data, OverviewDimension::Model),
        dimension(&axis, data, OverviewDimension::Client),
        tokens(&axis, series),
        work(&axis, series),
        latency(&axis, series),
    ]
}

fn volume(axis: &Axis, s: &[OverviewSeriesPoint]) -> SvgLineChartView {
    axis.chart(Plot::new(
        "Requests, conversations and people",
        format!(
            "{} requests · {} conversation-buckets · peak {} people in one bucket",
            short_num(sum(s, |p| p.requests)),
            short_num(sum(s, |p| p.conversations)),
            peak(s, |p| p.users)
        ),
        vec![
            input(
                s,
                "Requests",
                |p| p.requests,
                short_num(sum(s, |p| p.requests)),
            ),
            input(
                s,
                "Conversations",
                |p| p.conversations,
                short_num(sum(s, |p| p.conversations)),
            ),
            input(s, "People", |p| p.users, peak(s, |p| p.users).to_string()),
        ],
    ))
}

fn tokens(axis: &Axis, s: &[OverviewSeriesPoint]) -> SvgLineChartView {
    let total = sum(s, |p| p.tokens);
    let cached = sum(s, |p| p.cache_tokens);
    axis.chart(Plot {
        y_unit: "k tok",
        ..Plot::new(
            "Tokens and cache reads",
            format!(
                "{} input + output · {} served from cache",
                format_token_total(total),
                format_token_total(cached)
            ),
            vec![
                input(
                    s,
                    "Tokens (k)",
                    |p| p.tokens / 1000,
                    format_token_total(total),
                ),
                input(
                    s,
                    "Cache reads (k)",
                    |p| p.cache_tokens / 1000,
                    format_token_total(cached),
                ),
            ],
        )
    })
}

fn work(axis: &Axis, s: &[OverviewSeriesPoint]) -> SvgLineChartView {
    axis.chart(Plot::new(
        "Tool calls, errors and denials",
        format!(
            "{} tool calls · {} failed requests · {} denied by governance · {} rejected before routing",
            short_num(sum(s, |p| p.tool_calls)),
            sum(s, |p| p.errors),
            sum(s, |p| p.denied),
            sum(s, |p| p.rejected)
        ),
        vec![
            input(
                s,
                "Tool calls",
                |p| p.tool_calls,
                short_num(sum(s, |p| p.tool_calls)),
            ),
            input(s, "Errors", |p| p.errors, sum(s, |p| p.errors).to_string()),
            input(s, "Denied", |p| p.denied, sum(s, |p| p.denied).to_string()),
            input(s, "Rejected", |p| p.rejected, sum(s, |p| p.rejected).to_string()),
        ],
    ))
}

fn latency(axis: &Axis, s: &[OverviewSeriesPoint]) -> SvgLineChartView {
    axis.chart(Plot {
        y_unit: "ms",
        y_display: short_num,
        ..Plot::new(
            "Latency per bucket",
            format!(
                "worst bucket p95 {} · worst bucket p50 {}",
                latency_display(peak(s, |p| p.p95_latency_ms)),
                latency_display(peak(s, |p| p.p50_latency_ms))
            ),
            vec![
                input(
                    s,
                    "p95",
                    |p| p.p95_latency_ms,
                    latency_display(peak(s, |p| p.p95_latency_ms)),
                ),
                input(
                    s,
                    "p50",
                    |p| p.p50_latency_ms,
                    latency_display(peak(s, |p| p.p50_latency_ms)),
                ),
            ],
        )
    })
}

fn latency_display(ms: i64) -> String {
    if ms >= 1000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{ms}ms")
    }
}

// Why: pivots the sparse (bucket, key) rows onto the spine, one series per
// key ranked by its window total so the biggest spender takes the first
// colour — the same order the model board lists them in.
fn dimension(axis: &Axis, data: &OverviewData, dim: OverviewDimension) -> SvgLineChartView {
    let (rows, title, measure): (
        &[DimensionBucket],
        &'static str,
        fn(&DimensionBucket) -> i64,
    ) = match dim {
        OverviewDimension::Model => (
            data.model_series.as_deref().unwrap_or(&[]),
            "Spend by model",
            |r| r.cost_microdollars,
        ),
        OverviewDimension::Client => (
            data.client_series.as_deref().unwrap_or(&[]),
            "Requests by client",
            |r| r.requests,
        ),
    };
    let n = axis.labels.len();
    let mut keys: Vec<(String, i64)> = Vec::new();
    for row in rows {
        match keys.iter_mut().find(|(k, _)| *k == row.key) {
            Some((_, total)) => *total += measure(row),
            None => keys.push((row.key.clone(), measure(row))),
        }
    }
    keys.sort_by_key(|(_, total)| std::cmp::Reverse(*total));
    let series = keys
        .iter()
        .map(|(key, total)| {
            let mut values = vec![0i64; n];
            for row in rows.iter().filter(|r| r.key == *key) {
                if let Some(slot) = values.get_mut(row.bucket) {
                    *slot += measure(row);
                }
            }
            SvgSeriesInput {
                label: dimension_label(dim, key),
                values,
                value_display: dimension_display(dim, *total),
            }
        })
        .collect();
    let grand: i64 = keys.iter().map(|(_, t)| *t).sum();
    let plot = match dim {
        OverviewDimension::Model => Plot {
            y_unit: "µ$",
            y_display: format_cost,
            ..Plot::new(
                title,
                format!("{} across {} models", format_cost(grand), keys.len()),
                series,
            )
        },
        OverviewDimension::Client => Plot::new(
            title,
            format!("{} requests from {} clients", short_num(grand), keys.len()),
            series,
        ),
    };
    axis.chart(plot).into_columns()
}

fn dimension_label(dim: OverviewDimension, key: &str) -> String {
    match (dim, key) {
        (_, "Other") => "Other".to_owned(),
        (OverviewDimension::Model, "unrouted") => "Unrouted".to_owned(),
        (OverviewDimension::Model, id) => short_name(id, '/'),
        (OverviewDimension::Client, kind) => client_label(kind),
    }
}

fn dimension_display(dim: OverviewDimension, total: i64) -> String {
    match dim {
        OverviewDimension::Model => format_cost(total),
        OverviewDimension::Client => short_num(total),
    }
}
