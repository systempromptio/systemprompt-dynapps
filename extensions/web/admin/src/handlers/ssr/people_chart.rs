//! The thirty-day request chart on the group detail page.

use systemprompt_web_admin_core::ssr::format::short_num;
use systemprompt_web_admin_core::ssr::types::svg_line::{
    LineChartSpec, SvgLineChartView, SvgSeriesInput, line_chart,
};

pub(crate) fn daily_requests_chart(daily: &[i64]) -> SvgLineChartView {
    let total: i64 = daily.iter().sum();
    line_chart(LineChartSpec {
        title: "Daily volume",
        subtitle: format!("{} requests", short_num(total)),
        empty_message: "No gateway traffic in the last 30 days.",
        series: vec![SvgSeriesInput {
            label: "Requests".to_owned(),
            values: daily.to_vec(),
            value_display: short_num(total),
        }],
        ref_lines: Vec::new(),
        y_max: None,
        y_display: |v| v.to_string(),
        x_start_display: "30d ago".to_owned(),
        x_mid_display: "15d ago".to_owned(),
        x_end_display: "today".to_owned(),
        show_area: true,
        x_labels: Vec::new(),
        y_unit: "",
    })
}
