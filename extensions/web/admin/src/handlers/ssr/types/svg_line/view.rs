//! The serialized shape of the `svg-line-chart` partial: every value is
//! display-ready, so the template compares and prints but never computes.

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ChartKind {
    Line,
    Stacked,
}

#[derive(Debug, Serialize)]
pub(crate) struct SvgLineChartView {
    // Why: Handlebars — the layout partial's `title=` hash param shadows a
    // context field named `title` inside nested partials.
    #[serde(rename = "chart_title")]
    pub title: &'static str,
    pub subtitle: String,
    pub has_data: bool,
    pub empty_message: &'static str,
    pub aria_label: String,
    pub series: Vec<SvgSeriesView>,
    pub ref_lines: Vec<SvgRefLineView>,
    pub legend: Vec<SvgLegendItemView>,
    pub y_max_display: String,
    pub y_mid_display: String,
    pub x_start_display: String,
    pub x_mid_display: String,
    pub x_end_display: String,
    pub x_labels_json: String,
    pub y_unit: &'static str,
    pub kind: ChartKind,
}

impl SvgLineChartView {
    pub(crate) const fn into_columns(mut self) -> Self {
        self.kind = ChartKind::Stacked;
        self
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct SvgSeriesView {
    pub path_d: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub area_d: Option<String>,
    pub color_token: &'static str,
    pub label: String,
    pub values: String,
    pub value_display: String,
    pub points: Vec<SvgPointView>,
}

#[derive(Debug, Serialize)]
pub(crate) struct SvgPointView {
    pub x: String,
    pub y: String,
    // Why: Handlebars strict mode rejects `../this`, so the series' colour is
    // repeated on each dot rather than read from the parent context.
    pub color_token: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct SvgRefLineView {
    pub y: String,
    pub label: String,
    pub tone: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct SvgLegendItemView {
    pub label: String,
    pub color_index: usize,
    pub value_display: String,
}
