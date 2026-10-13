//! The Cost tab: every gateway request the run's session made, with totals,
//! and the per-turn cost as a bar chart drawn by `components/timeseries-chart`.

use serde::Serialize;
use systemprompt_web_admin_core::ssr::format::{
    format_cost, format_duration_ms, local_time, short_id, short_num,
};

use crate::repositories::summaries::RunRequestRow;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RunCostView {
    pub rows: Vec<CostRowView>,
    pub has_rows: bool,
    pub total_cost: String,
    pub total_tokens_in: String,
    pub total_tokens_out: String,
    pub chart: CostChartView,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CostRowView {
    pub index: usize,
    pub request_id: String,
    pub request_id_short: String,
    pub url: String,
    pub model: String,
    pub latency: String,
    pub tokens_in: String,
    pub tokens_out: String,
    pub cost: String,
    pub created_local: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CostChartView {
    // Why: serialized as chart_title — the layout partial's `title=` hash
    // would otherwise shadow it inside components/timeseries-chart.
    #[serde(rename = "chart_title")]
    pub title: &'static str,
    pub subtitle: String,
    pub tone: &'static str,
    pub series: Vec<CostBar>,
    pub has_data: bool,
    pub y_max_display: String,
    pub y_mid_display: String,
    pub x_start_display: String,
    pub x_mid_display: String,
    pub x_end_display: String,
    pub empty_message: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CostBar {
    pub pct: f64,
    pub tooltip: String,
}

pub(crate) fn cost_view(requests: &[RunRequestRow]) -> RunCostView {
    let rows: Vec<CostRowView> = requests
        .iter()
        .enumerate()
        .map(|(i, r)| CostRowView {
            index: i + 1,
            request_id: r.request_id.clone(),
            request_id_short: short_id(&r.request_id),
            url: format!("/admin/requests/{}", r.request_id),
            model: r.model.clone().unwrap_or_else(|| "—".to_owned()),
            latency: r
                .latency_ms
                .map_or_else(|| "—".to_owned(), |ms| format_duration_ms(i64::from(ms))),
            tokens_in: short_num(r.input_tokens.map_or(0, i64::from)),
            tokens_out: short_num(r.output_tokens.map_or(0, i64::from)),
            cost: format_cost(r.cost_microdollars),
            created_local: local_time(r.created_at),
        })
        .collect();
    let sum = |f: fn(&RunRequestRow) -> i64| requests.iter().map(f).sum::<i64>();
    RunCostView {
        has_rows: !rows.is_empty(),
        total_cost: format_cost(sum(|r| r.cost_microdollars)),
        total_tokens_in: short_num(sum(|r| r.input_tokens.map_or(0, i64::from))),
        total_tokens_out: short_num(sum(|r| r.output_tokens.map_or(0, i64::from))),
        chart: chart(requests),
        rows,
    }
}

fn chart(requests: &[RunRequestRow]) -> CostChartView {
    let max = requests
        .iter()
        .map(|r| r.cost_microdollars)
        .max()
        .unwrap_or(0);
    let n = requests.len();
    let series = requests
        .iter()
        .enumerate()
        .map(|(i, r)| CostBar {
            pct: if max > 0 {
                (r.cost_microdollars as f64 / max as f64 * 1000.0).round() / 10.0
            } else {
                0.0
            },
            tooltip: format!("Turn {} · {}", i + 1, format_cost(r.cost_microdollars)),
        })
        .collect();
    CostChartView {
        title: "Cost per turn",
        subtitle: format!("{n} gateway requests, in order"),
        tone: "accent",
        series,
        has_data: max > 0,
        y_max_display: format_cost(max),
        y_mid_display: format_cost(max / 2),
        x_start_display: "turn 1".to_owned(),
        x_mid_display: if n > 2 {
            format!("turn {}", n.div_ceil(2))
        } else {
            String::new()
        },
        x_end_display: format!("turn {}", n.max(1)),
        empty_message: "No billed gateway requests recorded for this run yet.",
    }
}
