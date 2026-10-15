//! Outcome figures shared by the overview, the clients list and one client:
//! the KPI strip and the two trend charts, all drawn from the same zero-filled
//! daily series ([`DailyOutcomeRow`]) so a tile and the chart under it can
//! never disagree. Rates and scores are 0–100; a day with nothing finished
//! carries the previous day's figure forward so a line shows where a rate
//! stands rather than dropping to zero between runs.

use chrono::NaiveDate;
use serde::Serialize;
use systemprompt_web_admin_core::ssr::format::short_num;
use systemprompt_web_admin_core::ssr::types::sparkline::sparkline_toned;
use systemprompt_web_admin_core::ssr::types::svg_line::{
    LineChartSpec, SvgLineChartView, SvgSeriesInput, line_chart,
};
use systemprompt_web_shared::format::format_cost;

use crate::handlers::ssr::farm_overview::activity::KpiTile;
use crate::repositories::clients::{DailyOutcomeRow, pooled_eval_mean};

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
pub(crate) struct OutcomeTotals {
    pub runs: i64,
    pub succeeded: i64,
    pub finished: i64,
    pub cost_microdollars: i64,
    pub eval_mean: Option<f64>,
    pub eval_scored: i64,
    pub infra_failures: i64,
    pub completed: Option<(i64, i64)>,
}

impl OutcomeTotals {
    pub(crate) fn from_days(days: &[DailyOutcomeRow]) -> Self {
        Self {
            runs: days.iter().map(|d| d.runs).sum(),
            succeeded: days.iter().map(|d| d.succeeded).sum(),
            finished: days.iter().map(|d| d.finished).sum(),
            cost_microdollars: days.iter().map(|d| d.cost_microdollars).sum(),
            eval_mean: pooled_eval_mean(days),
            eval_scored: days.iter().map(|d| d.eval_scored).sum(),
            infra_failures: days.iter().map(|d| d.infra_failures).sum(),
            completed: None,
        }
    }

    pub(crate) fn success_pct(&self) -> Option<f64> {
        (self.finished > 0).then(|| self.succeeded as f64 * 100.0 / self.finished as f64)
    }
}

pub(crate) fn pct_display(value: Option<f64>) -> String {
    value.map_or_else(|| "—".to_owned(), |v| format!("{v:.0}%"))
}

pub(crate) fn score_display(value: Option<f64>) -> String {
    value.map_or_else(|| "—".to_owned(), |v| format!("{v:.0}"))
}

pub(crate) fn rate_tone(pct: Option<f64>) -> &'static str {
    match pct {
        Some(p) if p >= 90.0 => "ok",
        Some(p) if p >= 70.0 => "warn",
        Some(_) => "err",
        None => "",
    }
}

pub(crate) const fn tile(
    key: &'static str,
    label: &'static str,
    icon: &'static str,
    value: String,
    note: String,
) -> KpiTile {
    KpiTile {
        key,
        label,
        icon,
        value,
        note,
        hint: String::new(),
        href: None,
        tone: "",
        spark: None,
    }
}

pub(crate) fn outcome_kpis(
    totals: &OutcomeTotals,
    days: &[DailyOutcomeRow],
    window: &str,
) -> Vec<KpiTile> {
    let daily: Vec<i64> = days.iter().map(|d| d.runs).collect();
    let rate = totals.success_pct();
    let mut tiles = vec![
        KpiTile {
            spark: Some(sparkline_toned(
                &daily,
                "accent",
                format!("Runs per day, last {window}"),
            )),
            ..tile(
                "runs",
                "Runs",
                "bolt",
                short_num(totals.runs),
                format!("last {window}"),
            )
        },
        KpiTile {
            tone: rate_tone(rate),
            hint: format!(
                "{} of {} finished runs succeeded",
                totals.succeeded, totals.finished
            ),
            ..tile(
                "success",
                "Success rate",
                "check",
                pct_display(rate),
                format!("{} finished", totals.finished),
            )
        },
        tile(
            "cost",
            "Cost",
            "coins",
            format_cost(totals.cost_microdollars),
            format!("{} runs", short_num(totals.runs)),
        ),
        KpiTile {
            tone: rate_tone(totals.eval_mean),
            hint: "Infrastructure failures (host crash, connector unavailable, eval errored) are left out of the mean".to_owned(),
            ..tile(
                "eval",
                "Eval score",
                "judge",
                score_display(totals.eval_mean),
                format!(
                    "{} scored · {} infra failures",
                    totals.eval_scored, totals.infra_failures
                ),
            )
        },
    ];
    if let Some((completed, reported)) = totals.completed {
        let pct = (reported > 0).then(|| completed as f64 * 100.0 / reported as f64);
        tiles.push(KpiTile {
            tone: rate_tone(pct),
            ..tile(
                "completed",
                "Task completion",
                "report",
                pct_display(pct),
                format!("{completed} of {reported} reported runs"),
            )
        });
    }
    tiles
}

fn axis_spec(
    title: &'static str,
    subtitle: String,
    series: Vec<SvgSeriesInput>,
    days: &[DailyOutcomeRow],
    pinned: Option<i64>,
) -> LineChartSpec {
    let label = |d: Option<&NaiveDate>| {
        d.map(|d| d.format("%b %-d").to_string())
            .unwrap_or_default()
    };
    LineChartSpec {
        title,
        subtitle,
        empty_message: "No farm runs in this window.",
        series,
        ref_lines: Vec::new(),
        y_max: pinned,
        y_display: |v| v.to_string(),
        x_start_display: label(days.first().map(|d| &d.day)),
        x_mid_display: label(days.get(days.len() / 2).map(|d| &d.day)),
        x_end_display: label(days.last().map(|d| &d.day)),
        show_area: true,
        x_labels: days
            .iter()
            .map(|d| d.day.format("%b %-d").to_string())
            .collect(),
        y_unit: if pinned.is_some() { "%" } else { "" },
    }
}

pub(crate) fn runs_chart(days: &[DailyOutcomeRow], window: &str) -> SvgLineChartView {
    let total: i64 = days.iter().map(|d| d.runs).sum();
    let ok: i64 = days.iter().map(|d| d.succeeded).sum();
    let series = if total == 0 {
        Vec::new()
    } else {
        vec![
            SvgSeriesInput {
                label: "Runs".to_owned(),
                values: days.iter().map(|d| d.runs).collect(),
                value_display: total.to_string(),
            },
            SvgSeriesInput {
                label: "Succeeded".to_owned(),
                values: days.iter().map(|d| d.succeeded).collect(),
                value_display: ok.to_string(),
            },
        ]
    };
    line_chart(axis_spec(
        "Runs over time",
        format!("Runs created per day and how many succeeded, last {window}"),
        series,
        days,
        None,
    ))
}

fn carried(values: impl Iterator<Item = Option<f64>>) -> Vec<i64> {
    let values: Vec<Option<f64>> = values.collect();
    let mut last = values.iter().flatten().next().copied().unwrap_or(0.0);
    values
        .into_iter()
        .map(|v| {
            if let Some(v) = v {
                last = v;
            }
            last.round() as i64
        })
        .collect()
}

pub(crate) fn rates_chart(days: &[DailyOutcomeRow], window: &str) -> SvgLineChartView {
    let mut series = Vec::new();
    if days.iter().any(|d| d.finished > 0) {
        let values = carried(
            days.iter()
                .map(|d| (d.finished > 0).then(|| d.succeeded as f64 * 100.0 / d.finished as f64)),
        );
        let latest = values.last().copied().unwrap_or(0);
        series.push(SvgSeriesInput {
            label: "Success rate".to_owned(),
            values,
            value_display: format!("{latest}%"),
        });
    }
    if days.iter().any(|d| d.eval_mean.is_some()) {
        let values = carried(days.iter().map(|d| d.eval_mean));
        let latest = values.last().copied().unwrap_or(0);
        series.push(SvgSeriesInput {
            label: "Eval score".to_owned(),
            values,
            value_display: latest.to_string(),
        });
    }
    line_chart(axis_spec(
        "Success and eval trend",
        format!("Daily success rate and mean eval score, 0–100, last {window}"),
        series,
        days,
        Some(100),
    ))
}
