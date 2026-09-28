//! Server-rendered SVG line-chart view-models; sparklines and deltas are in
//! `sparkline`.
//!
//! All geometry comes from [`crate::util::svg`]; this module only shapes it
//! for the `svg-line-chart` partial and the analytics sparklines. Same design
//! rule as `charts.rs`: the view carries every derived value (paths, axis
//! labels, reference-line positions, the legend's per-series figure), so the
//! template does no arithmetic and the axis cannot disagree with the lines.
//! The colour tokens are shared with the pie so a model keeps one colour
//! across every chart in a window, and an area fill is drawn for a single
//! series only, because a stacked fill misstates the values beneath it. A
//! spec's `y_max` of `None` derives a round max from the data; `Some` pins it.

mod plot;
mod view;

use crate::util::svg;

pub(crate) use plot::{Plot, chart_on_axis};
use view::ChartKind;
pub(crate) use view::{
    SvgLegendItemView, SvgLineChartView, SvgPointView, SvgRefLineView, SvgSeriesView,
};

pub(crate) const CHART_COLOR_TOKENS: [&str; 7] = [
    "--sp-chart-purple",
    "--sp-chart-blue",
    "--sp-chart-green",
    "--sp-chart-amber",
    "--sp-chart-red",
    "--sp-chart-cyan",
    "--sp-chart-indigo",
];

const POINT_DOTS_MAX_BUCKETS: usize = 12;
const AREA_MIN_BUCKETS: usize = 3;

pub(crate) struct SvgSeriesInput {
    pub label: String,
    pub values: Vec<i64>,
    pub value_display: String,
}

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
    pub show_area: bool,
    pub x_labels: Vec<String>,
    pub y_unit: &'static str,
}

fn point_dots(points: &[(f64, f64)], color_token: &'static str) -> Vec<SvgPointView> {
    if points.len() > POINT_DOTS_MAX_BUCKETS {
        return Vec::new();
    }
    points
        .iter()
        .map(|(x, y)| SvgPointView {
            x: format!("{x:.2}"),
            y: format!("{y:.2}"),
            color_token,
        })
        .collect()
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
            let color_token = CHART_COLOR_TOKENS[i.min(CHART_COLOR_TOKENS.len() - 1)];
            let dots = point_dots(&points, color_token);
            SvgSeriesView {
                path_d: svg::line_path(&points),
                area_d: (spec.show_area && !multi && points.len() >= AREA_MIN_BUCKETS)
                    .then(|| svg::area_path(&points)),
                points: dots,
                color_token,
                label: s.label.clone(),
                value_display: s.value_display.clone(),
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
        kind: ChartKind::Line,
    }
}

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
