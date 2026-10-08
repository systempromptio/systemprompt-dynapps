//! A chart's own part of a line-chart spec — everything but the shared x
//! axis — so a page drawing several charts on one axis passes one `Plot` each.

use super::{LineChartSpec, SvgLineChartView, SvgSeriesInput, line_chart};

#[derive(Debug)]
pub struct Plot {
    pub title: &'static str,
    pub subtitle: String,
    pub series: Vec<SvgSeriesInput>,
    pub ref_lines: Vec<(i64, String, &'static str)>,
    pub y_max: Option<i64>,
    pub y_unit: &'static str,
    pub y_display: fn(i64) -> String,
}

impl Plot {
    pub fn new(title: &'static str, subtitle: String, series: Vec<SvgSeriesInput>) -> Self {
        Self {
            title,
            subtitle,
            series,
            ref_lines: Vec::new(),
            y_max: None,
            y_unit: "",
            y_display: |v| v.to_string(),
        }
    }
}

pub fn chart_on_axis(
    labels: &[String],
    empty_message: &'static str,
    plot: Plot,
) -> SvgLineChartView {
    let n = labels.len();
    line_chart(LineChartSpec {
        title: plot.title,
        subtitle: plot.subtitle,
        empty_message,
        series: plot.series,
        ref_lines: plot.ref_lines,
        y_max: plot.y_max,
        y_display: plot.y_display,
        x_start_display: labels.first().cloned().unwrap_or_default(),
        x_mid_display: labels.get(n / 2).cloned().unwrap_or_default(),
        x_end_display: labels.last().cloned().unwrap_or_default(),
        show_area: true,
        x_labels: labels.to_vec(),
        y_unit: plot.y_unit,
    })
}
