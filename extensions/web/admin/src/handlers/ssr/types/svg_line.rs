//! Server-rendered SVG line-chart view-models; sparklines and deltas are in
//! `sparkline`.
//!
//! All geometry comes from [`crate::util::svg`]; this module only shapes it
//! for the `svg-line-chart` partial and the analytics sparklines. Same design
//! rule as `charts.rs`: the view carries every derived value (paths, axis
//! labels, reference-line positions), so the template does no arithmetic and
//! the axis cannot disagree with the lines.

use serde::Serialize;

use crate::util::svg;

// Why: shared with the pie so a model keeps one color across every chart in
// a window.
pub(crate) const CHART_COLOR_TOKENS: [&str; 7] = [
    "--sp-chart-purple",
    "--sp-chart-blue",
    "--sp-chart-green",
    "--sp-chart-amber",
    "--sp-chart-red",
    "--sp-chart-cyan",
    "--sp-chart-indigo",
];

#[derive(Debug, Serialize)]
pub(crate) struct SvgLineChartView {
    // Why: serialized as chart_title — the layout partial's `title=` hash
    // param shadows a context field named `title` inside nested partials.
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
    // Why: one label per bucket, JSON-encoded, for the client-side crosshair
    // (`sp-chart.js`) — the server keeps rendering the plot, the script only
    // narrates it on hover.
    pub x_labels_json: String,
    pub y_unit: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct SvgSeriesView {
    pub path_d: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub area_d: Option<String>,
    pub color_token: &'static str,
    pub label: String,
    // Why: the raw values behind the path, comma-separated, so the hover
    // tooltip prints numbers and never re-derives them from geometry.
    pub values: String,
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

// Why: one series' inputs before geometry — a label, its values already
// bucketed onto the shared spine, and the total the legend prints.
pub(crate) struct SvgSeriesInput {
    pub label: String,
    pub values: Vec<i64>,
    pub value_display: String,
}

// Why: `y_max` of `None` derives a round max from the data; `Some` pins it —
// the burn-up pins to the cap so the cap line always has room on the plot.
pub(crate) struct LineChartSpec {
    pub title: &'static str,
    pub subtitle: String,
    pub empty_message: &'static str,
    pub series: Vec<SvgSeriesInput>,
    pub ref_lines: Vec<(i64, String, &'static str)>,
    pub y_max: Option<i64>,
    pub y_display: fn(i64) -> String,
    pub x_start_display: String,
    pub x_mid_display: String,
    pub x_end_display: String,
    // Why: Fill under the line — single-series charts only (stacked fills lie).
    pub show_area: bool,
    // Why: one label per bucket for the hover crosshair; empty leaves the
    // chart static.
    pub x_labels: Vec<String>,
    // Why: the unit the tooltip appends to a value ("", "$", "ms", "tok").
    pub y_unit: &'static str,
}

pub(crate) fn line_chart(spec: LineChartSpec) -> SvgLineChartView {
    let data_max = spec
        .series
        .iter()
        .flat_map(|s| s.values.iter().copied())
        .max()
        .unwrap_or(0);
    let y_max = spec
        .y_max
        .map_or_else(|| svg::nice_max(data_max), |m| m.max(1));
    let has_data = data_max > 0;
    let multi = spec.series.len() > 1;

    let series: Vec<SvgSeriesView> = spec
        .series
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let points = svg::scale_points(&s.values, y_max);
            SvgSeriesView {
                path_d: svg::line_path(&points),
                area_d: (spec.show_area && !multi).then(|| svg::area_path(&points)),
                color_token: CHART_COLOR_TOKENS[i.min(CHART_COLOR_TOKENS.len() - 1)],
                label: s.label.clone(),
                values: s
                    .values
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            }
        })
        .collect();

    let ref_lines = spec
        .ref_lines
        .iter()
        .filter_map(|(value, label, tone)| {
            svg::ref_line_y(*value, y_max).map(|y| SvgRefLineView {
                y: format!("{y:.2}"),
                label: label.clone(),
                tone,
            })
        })
        .collect();

    let legend = legend_for(&spec.series, multi);
    let aria_label = aria_label_for(&spec, data_max);

    SvgLineChartView {
        title: spec.title,
        subtitle: spec.subtitle,
        has_data,
        empty_message: spec.empty_message,
        aria_label,
        series,
        ref_lines,
        legend,
        y_max_display: (spec.y_display)(y_max),
        y_mid_display: (spec.y_display)(y_max / 2),
        x_start_display: spec.x_start_display,
        x_mid_display: spec.x_mid_display,
        x_end_display: spec.x_end_display,
        x_labels_json: serde_json::to_string(&spec.x_labels).unwrap_or_else(|_| "[]".to_owned()),
        y_unit: spec.y_unit,
    }
}

// Why: a chart's own part of a spec — everything but the shared x axis — so
// a page drawing several charts on one axis passes one value per chart.
pub(crate) struct Plot {
    pub title: &'static str,
    pub subtitle: String,
    pub series: Vec<SvgSeriesInput>,
    pub ref_lines: Vec<(i64, String, &'static str)>,
    pub y_max: Option<i64>,
    pub y_unit: &'static str,
    pub y_display: fn(i64) -> String,
}

impl Plot {
    pub(crate) fn new(title: &'static str, subtitle: String, series: Vec<SvgSeriesInput>) -> Self {
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

// Why: draws one plot on a shared axis: the first, middle and last labels go
// to the gutters, every label to the hover layer.
pub(crate) fn chart_on_axis(
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

// Why: a legend only earns its place with two or more series.
fn legend_for(series: &[SvgSeriesInput], multi: bool) -> Vec<SvgLegendItemView> {
    if !multi {
        return Vec::new();
    }
    series
        .iter()
        .enumerate()
        .map(|(i, s)| SvgLegendItemView {
            label: s.label.clone(),
            color_index: i.min(CHART_COLOR_TOKENS.len() - 1) + 1,
            value_display: s.value_display.clone(),
        })
        .collect()
}

fn aria_label_for(spec: &LineChartSpec, data_max: i64) -> String {
    format!(
        "{}: {} — peak {}",
        spec.title,
        spec.series
            .iter()
            .map(|s| format!("{} {}", s.label, s.value_display))
            .collect::<Vec<_>>()
            .join(", "),
        (spec.y_display)(data_max)
    )
}
