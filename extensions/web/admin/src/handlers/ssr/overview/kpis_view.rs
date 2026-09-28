//! The KPI strip: ten figures, each icon-led, toned, carrying the window's
//! own sparkline in its trend slot, its window-over-window delta, and a link
//! to the page that owns the rows it counted.
//!
//! The tile lists themselves live in `kpi_tiles`; this module owns the strip,
//! the tile shape and the shared arithmetic, so a figure and its delta are
//! derived from one integer and cannot round apart.

use serde::Serialize;

use crate::repositories::overview::series::OverviewSeriesPoint;
use systemprompt_web_admin_core::ssr::types::sparkline::{SparklineView, sparkline_toned};
use systemprompt_web_admin_core::util::delta::Delta;

use super::data::OverviewData;
use super::kpi_tiles::{health_tiles, volume_tiles, work_tiles};
use super::view::OverviewRange;

#[derive(Debug, Serialize)]
pub(super) struct OverviewKpiStripView {
    pub error: Option<String>,
    pub comparison_note: Option<String>,
    pub tiles: Vec<KpiTileView>,
}

#[derive(Debug, Serialize)]
pub(super) struct KpiTileView {
    pub label: &'static str,
    pub icon: &'static str,
    pub value: String,
    pub note: String,
    pub hint: String,
    pub href: String,
    pub tone: &'static str,
    pub delta: String,
    pub delta_dir: &'static str,
    pub spark: SparklineView,
}

const COMPARABLE_PRIOR_REQUESTS: i64 = 100;

pub(super) struct TileInputs<'a> {
    pub range: OverviewRange,
    pub comparable: bool,
    pub series: &'a [OverviewSeriesPoint],
}

impl TileInputs<'_> {
    pub(super) fn column(&self, f: fn(&OverviewSeriesPoint) -> i64) -> Vec<i64> {
        self.series.iter().map(f).collect()
    }
}

pub(super) fn kpi_strip(range: OverviewRange, data: &OverviewData) -> OverviewKpiStripView {
    let k = data.kpis.as_ref().copied().unwrap_or_default();
    let comparable = k.prev_requests >= COMPARABLE_PRIOR_REQUESTS;
    let inputs = TileInputs {
        range,
        comparable,
        series: data.series.as_deref().unwrap_or(&[]),
    };
    OverviewKpiStripView {
        error: data
            .kpis
            .as_ref()
            .err()
            .map(|e| format!("The headline figures could not be read: {e}")),
        comparison_note: (!comparable).then(|| {
            format!(
                "The previous window carried {} requests, too few to compare against, so no tile shows a percentage.",
                k.prev_requests
            )
        }),
        tiles: volume_tiles(&k, &inputs)
            .into_iter()
            .chain(work_tiles(&k, &inputs))
            .chain(health_tiles(&k, &inputs))
            .collect(),
    }
}

pub(super) struct Tile {
    pub label: &'static str,
    pub icon: &'static str,
    pub value: String,
    pub note: String,
    pub hint: String,
    pub href: String,
    pub tone: &'static str,
}

pub(super) fn tile(spec: Tile, d: Option<Delta>, spark_values: &[i64]) -> KpiTileView {
    let spark_tone = match spec.tone {
        "" => "accent",
        tone => tone,
    };
    KpiTileView {
        spark: sparkline_toned(
            spark_values,
            spark_tone,
            format!("{} per bucket over the window", spec.label),
        ),
        label: spec.label,
        icon: spec.icon,
        value: spec.value,
        note: spec.note,
        hint: spec.hint,
        href: spec.href,
        tone: spec.tone,
        delta: d.map(|d| d.display()).unwrap_or_default(),
        delta_dir: d.map_or("", delta_dir),
    }
}

pub(super) fn comparison(prior: &str, vs: &str) -> String {
    format!("{prior} {vs}")
}

fn delta_dir(d: Delta) -> &'static str {
    match (d.direction, d.tone) {
        ("up", "bad") => "up-bad",
        ("down", "good") => "down-good",
        ("up", _) => "up",
        ("down", _) => "down",
        _ => "",
    }
}

pub(super) fn percent_tenths(tenths: i64, sample: i64) -> String {
    if sample <= 0 {
        return "\u{2014}".to_owned();
    }
    format!("{}.{}%", tenths / 10, tenths % 10)
}
