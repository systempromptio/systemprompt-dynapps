//! The KPI tiles and the four day-by-day charts of the skill detail page;
//! the tiles are in `tiles`, and the charts share one day axis.

mod tiles;

use crate::repositories::analysis::skills::SkillDayRow;
use systemprompt_web_admin_core::ssr::format::{format_cost, format_token_total};
use systemprompt_web_admin_core::ssr::types::svg_line::{
    Plot, SvgLineChartView, SvgSeriesInput, chart_on_axis,
};

pub(super) use tiles::kpis;

struct DayAxis {
    labels: Vec<String>,
}

impl DayAxis {
    fn new(daily: &[SkillDayRow]) -> Self {
        Self {
            labels: daily
                .iter()
                .map(|d| d.day.format("%b %-d").to_string())
                .collect(),
        }
    }

    fn chart(&self, plot: Plot) -> SvgLineChartView {
        chart_on_axis(&self.labels, "Nothing in this window", plot)
    }
}

fn series(
    daily: &[SkillDayRow],
    label: &str,
    f: fn(&SkillDayRow) -> i64,
    total: String,
) -> SvgSeriesInput {
    SvgSeriesInput {
        label: label.to_owned(),
        values: daily.iter().map(f).collect(),
        value_display: total,
    }
}

fn sum(daily: &[SkillDayRow], f: fn(&SkillDayRow) -> i64) -> i64 {
    daily.iter().map(f).sum()
}

fn volume_chart(axis: &DayAxis, daily: &[SkillDayRow]) -> SvgLineChartView {
    axis.chart(Plot::new(
        "Invocations and people per day",
        format!("{} invocations", sum(daily, |d| d.invocations)),
        vec![
            series(
                daily,
                "Invocations",
                |d| d.invocations,
                sum(daily, |d| d.invocations).to_string(),
            ),
            series(
                daily,
                "People",
                |d| d.users,
                daily.iter().map(|d| d.users).max().unwrap_or(0).to_string(),
            ),
            series(
                daily,
                "Conversations",
                |d| d.conversations,
                sum(daily, |d| d.conversations).to_string(),
            ),
        ],
    ))
}

fn spend_chart(axis: &DayAxis, daily: &[SkillDayRow]) -> SvgLineChartView {
    axis.chart(Plot::new(
        "Cost and tokens per day",
        format!(
            "{} · {} tokens",
            format_cost(sum(daily, |d| d.cost_microdollars)),
            format_token_total(sum(daily, |d| d.tokens))
        ),
        vec![
            series(
                daily,
                "Cost (µ$)",
                |d| d.cost_microdollars,
                format_cost(sum(daily, |d| d.cost_microdollars)),
            ),
            series(
                daily,
                "Tokens (k)",
                |d| d.tokens / 1000,
                format_token_total(sum(daily, |d| d.tokens)),
            ),
        ],
    ))
}

fn health_chart(axis: &DayAxis, daily: &[SkillDayRow]) -> SvgLineChartView {
    let worst = daily
        .iter()
        .filter_map(|d| d.p95_latency_ms)
        .fold(0.0f64, f64::max)
        .round();
    axis.chart(Plot {
        ref_lines: vec![(20_000, "20 s target".to_owned(), "warn")],
        ..Plot::new(
            "p95 latency and errors per day",
            "Gateway p95 per day against the 20 s target".to_owned(),
            vec![
                series(
                    daily,
                    "p95 (ms)",
                    |d| d.p95_latency_ms.map_or(0, |v| v.round() as i64),
                    worst.to_string(),
                ),
                series(
                    daily,
                    "Errors",
                    |d| d.errors,
                    sum(daily, |d| d.errors).to_string(),
                ),
            ],
        )
    })
}

fn completion_chart(axis: &DayAxis, daily: &[SkillDayRow]) -> SvgLineChartView {
    axis.chart(Plot {
        ref_lines: vec![
            (80, "good".to_owned(), "ok"),
            (50, "watch".to_owned(), "warn"),
        ],
        y_max: Some(100),
        ..Plot::new(
            "Completion per day",
            "Mean judge completion of the conversations that used the skill".to_owned(),
            vec![series(
                daily,
                "Completion",
                |d| d.completion_avg.map_or(0, |v| v.round() as i64),
                String::new(),
            )],
        )
    })
}

pub(super) fn charts(daily: &[SkillDayRow]) -> Vec<SvgLineChartView> {
    let axis = DayAxis::new(daily);
    vec![
        volume_chart(&axis, daily),
        spend_chart(&axis, daily),
        health_chart(&axis, daily),
        completion_chart(&axis, daily),
    ]
}
