//! The stacked cost-by-model chart: the one view that combines the model mix
//! with the cost series, so a spend spike can be attributed to a model without
//! reading two charts side by side.
//!
//! Split from `view.rs` at the 300-line ceiling.

use crate::repositories::analytics::site::model_series::ModelCostBucket;
use crate::repositories::analytics::site::series::UsageBucket;
use systemprompt_web_admin_core::ssr::format::format_cost;
use systemprompt_web_admin_core::ssr::types::pie::PieView;
use systemprompt_web_admin_core::ssr::types::svg_line::{
    Plot, SvgLineChartView, SvgSeriesInput, chart_on_axis,
};

use super::view::{self, bucket_label};

pub(super) fn model_cost_stack(
    rows: &[ModelCostBucket],
    spine: &[UsageBucket],
    weekly: bool,
) -> SvgLineChartView {
    let bucket_labels: Vec<String> = spine
        .iter()
        .map(|b| bucket_label(b.bucket_start, weekly))
        .collect();
    let index_of =
        |ts: chrono::DateTime<chrono::Utc>| spine.iter().position(|b| b.bucket_start == ts);

    let mut models: Vec<String> = Vec::new();
    let mut totals: Vec<i64> = Vec::new();
    for row in rows {
        if let Some(i) = models.iter().position(|m| *m == row.model) {
            totals[i] += row.cost_microdollars;
        } else {
            models.push(row.model.clone());
            totals.push(row.cost_microdollars);
        }
    }
    let mut order: Vec<usize> = (0..models.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(totals[i]));

    let series = order
        .iter()
        .map(|&i| {
            let mut values = vec![0i64; spine.len()];
            for row in rows.iter().filter(|r| r.model == models[i]) {
                if let Some(b) = index_of(row.bucket_start) {
                    values[b] += row.cost_microdollars;
                }
            }
            SvgSeriesInput {
                label: models[i].clone(),
                values,
                value_display: format_cost(totals[i]),
            }
        })
        .collect();

    let grand_total: i64 = totals.iter().sum();
    chart_on_axis(
        &bucket_labels,
        "No billed requests in this window.",
        Plot {
            y_unit: "µ$",
            y_display: format_cost,
            ..Plot::new(
                "Cost by model",
                format!(
                    "{} across {} models",
                    format_cost(grand_total),
                    models.len()
                ),
                series,
            )
        },
    )
    .into_columns()
}

pub(super) struct OverviewCharts {
    pub volume: SvgLineChartView,
    pub cost: SvgLineChartView,
    pub model_pie: PieView,
    pub model_cost: SvgLineChartView,
}

pub(super) fn overview_charts(
    fetched: &super::data::AnalyticsDashboardData,
    query: &super::AnalyticsDashboardQuery,
    weekly: bool,
) -> OverviewCharts {
    OverviewCharts {
        volume: view::volume_chart(&fetched.series, weekly),
        cost: view::spend_chart(&fetched.series, weekly),
        model_pie: view::model_pie(&fetched.models, query),
        model_cost: model_cost_stack(&fetched.model_cost, &fetched.series, weekly),
    }
}
