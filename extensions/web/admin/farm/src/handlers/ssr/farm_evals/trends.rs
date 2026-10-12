//! The two trend charts on the evals page: mean judge score per suite and
//! the overall case pass rate, one bucket per day of the window. A day with
//! no finished eval run carries the previous day's figure forward, and the
//! days before a suite's first run take its first figure, so a line shows
//! where a score stands rather than dropping to zero between runs.

use std::collections::BTreeMap;

use chrono::{Duration, NaiveDate, Utc};
use systemprompt_web_admin_core::ssr::types::svg_line::{
    LineChartSpec, SvgLineChartView, SvgSeriesInput, line_chart,
};

use crate::repositories::rows::EvalRunRow;

const MAX_SERIES: usize = 8;

fn days_axis(days: u32) -> Vec<NaiveDate> {
    let today = Utc::now().date_naive();
    (0..i64::from(days))
        .rev()
        .map(|back| today - Duration::days(back))
        .collect()
}

fn carried(axis: &[NaiveDate], points: &BTreeMap<NaiveDate, (f64, u32)>) -> Vec<i64> {
    let first = points
        .values()
        .next()
        .map_or(0.0, |(sum, n)| sum / f64::from(*n));
    let mut last = first;
    axis.iter()
        .map(|day| {
            if let Some((sum, n)) = points.get(day) {
                last = sum / f64::from(*n);
            }
            last.round() as i64
        })
        .collect()
}

fn run_day(run: &EvalRunRow) -> NaiveDate {
    run.finished_at.unwrap_or(run.created_at).date_naive()
}

fn spec(
    title: &'static str,
    subtitle: String,
    series: Vec<SvgSeriesInput>,
    axis: &[NaiveDate],
) -> LineChartSpec {
    let label = |d: Option<&NaiveDate>| {
        d.map(|d| d.format("%b %-d").to_string())
            .unwrap_or_default()
    };
    LineChartSpec {
        title,
        subtitle,
        empty_message: "No finished eval runs in this window.",
        series,
        ref_lines: Vec::new(),
        y_max: Some(100),
        y_display: |v| v.to_string(),
        x_start_display: label(axis.first()),
        x_mid_display: label(axis.get(axis.len() / 2)),
        x_end_display: label(axis.last()),
        show_area: true,
        x_labels: axis
            .iter()
            .map(|d| d.format("%b %-d").to_string())
            .collect(),
        y_unit: "",
    }
}

pub(crate) fn score_chart(runs: &[EvalRunRow], days: u32) -> SvgLineChartView {
    let axis = days_axis(days);
    let mut by_suite: BTreeMap<&str, BTreeMap<NaiveDate, (f64, u32)>> = BTreeMap::new();
    for run in runs {
        let Some(score) = run.mean_score else {
            continue;
        };
        let entry = by_suite
            .entry(run.suite_id.as_str())
            .or_default()
            .entry(run_day(run))
            .or_insert((0.0, 0));
        entry.0 += score;
        entry.1 += 1;
    }
    let series = by_suite
        .iter()
        .take(MAX_SERIES)
        .map(|(suite, points)| {
            let values = carried(&axis, points);
            let latest = values.last().copied().unwrap_or(0);
            SvgSeriesInput {
                label: (*suite).to_owned(),
                values,
                value_display: latest.to_string(),
            }
        })
        .collect();
    line_chart(spec(
        "Mean judge score",
        format!("Per suite, 0–100, last {days} days"),
        series,
        &axis,
    ))
}

pub(crate) fn pass_rate_chart(runs: &[EvalRunRow], days: u32) -> SvgLineChartView {
    let axis = days_axis(days);
    let mut by_day: BTreeMap<NaiveDate, (i64, i64)> = BTreeMap::new();
    for run in runs {
        let decided = i64::from(run.passed + run.failed + run.errored);
        if decided == 0 {
            continue;
        }
        let entry = by_day.entry(run_day(run)).or_insert((0, 0));
        entry.0 += i64::from(run.passed);
        entry.1 += decided;
    }
    let points: BTreeMap<NaiveDate, (f64, u32)> = by_day
        .into_iter()
        .map(|(day, (passed, decided))| (day, (passed as f64 * 100.0 / decided as f64, 1)))
        .collect();
    let series = if points.is_empty() {
        Vec::new()
    } else {
        let values = carried(&axis, &points);
        let latest = values.last().copied().unwrap_or(0);
        vec![SvgSeriesInput {
            label: "Pass rate".to_owned(),
            values,
            value_display: format!("{latest}%"),
        }]
    };
    line_chart(spec(
        "Case pass rate",
        format!("Share of decided cases that passed, last {days} days"),
        series,
        &axis,
    ))
}
