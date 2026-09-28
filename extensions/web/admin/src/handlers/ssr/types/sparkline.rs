//! Sparkline and delta view-models for KPI tiles and table rows.
//!
//! Same rule as the charts: geometry is computed here from
//! [`crate::util::svg`], the partial only prints attributes.

use serde::Serialize;

use crate::util::delta::{Delta, delta};
use crate::util::svg;

#[derive(Debug, Serialize)]
pub(crate) struct SparklineView {
    pub path_d: String,
    pub area_d: String,
    pub has_data: bool,
    pub tone: &'static str,
    pub last_x: String,
    pub last_y: String,
    pub title: String,
}

const SPARK_H: f64 = 24.0;

pub(crate) fn sparkline(values: &[i64]) -> SparklineView {
    sparkline_toned(values, "accent", String::new())
}

pub(crate) fn sparkline_toned(values: &[i64], tone: &'static str, title: String) -> SparklineView {
    let max = values.iter().copied().max().unwrap_or(0);
    if max <= 0 || values.len() < 2 {
        return SparklineView {
            path_d: String::new(),
            area_d: String::new(),
            has_data: false,
            tone,
            last_x: String::new(),
            last_y: String::new(),
            title,
        };
    }
    let points: Vec<(f64, f64)> = svg::scale_points(values, max)
        .into_iter()
        .map(|(x, y)| (x, y / svg::PLOT_H * SPARK_H))
        .collect();
    let (last_x, last_y) = points.last().copied().unwrap_or((0.0, 0.0));
    let area_d = format!(
        "{} L{:.2},{SPARK_H} L{:.2},{SPARK_H} Z",
        svg::line_path(&points),
        last_x,
        points.first().map_or(0.0, |p| p.0)
    );
    SparklineView {
        path_d: svg::line_path(&points),
        area_d,
        has_data: true,
        tone,
        last_x: format!("{last_x:.2}"),
        last_y: format!("{last_y:.2}"),
        title,
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct DeltaView {
    pub display: String,
    pub direction: &'static str,
    pub tone: &'static str,
}

pub(crate) fn delta_view(current: i64, previous: i64, up_is_good: bool) -> DeltaView {
    let d: Delta = delta(current, previous, up_is_good);
    DeltaView {
        display: d.display(),
        direction: d.direction,
        tone: d.tone,
    }
}
