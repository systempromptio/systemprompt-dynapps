//! The KPI strip: ten figures, each icon-led, toned, carrying the window's
//! own sparkline in its trend slot, its window-over-window delta, and a link
//! to the page that owns the rows it counted.
//!
//! The tile lists themselves live in `kpi_tiles`; this module owns the strip,
//! the tile shape and the shared arithmetic, so a figure and its delta are
//! derived from one integer and cannot round apart.

use serde::Serialize;

use crate::handlers::ssr::types::{SparklineView, sparkline_toned};
use crate::repositories::overview::series::OverviewSeriesPoint;
use crate::util::delta::Delta;

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
    pub sub: String,
    pub hint: String,
    pub href: String,
    pub tone: &'static str,
    pub delta: String,
    pub delta_dir: &'static str,
    pub spark: SparklineView,
}

// Why: a percentage against a nearly empty prior window is arithmetic, not
// information — 53 requests yesterday and 1,400 today reads as "+2537.7%",
// which tells a reader nothing they can act on and hides the one fact that
// matters, which is that there is barely a prior window to compare with. Below
// the floor the tiles say what the prior window held instead.
const COMPARABLE_PRIOR_REQUESTS: i64 = 100;

// Why: what every tile builder needs — the figures, the window, whether the
// prior window is worth a percentage, and the spine the sparklines are cut
// from — travelling as one value so the builders take one argument.
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
    // Why: one gate for all ten tiles rather than a floor per figure. Every
    // one of them is derived from the same prior window's traffic, so either
    // that window is worth comparing against or none of them are.
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

// Why: the tile's own fields travel together as one value so the builder takes
// three arguments rather than nine — the delta is the only thing computed
// separately, because its polarity is the caller's call and not the tile's.
pub(super) struct Tile {
    pub label: &'static str,
    pub icon: &'static str,
    pub value: String,
    pub sub: String,
    // Why: the figure a tile's own sub line has no room for. It reaches the
    // reader as the tile's title, so the sub line can stay one band and still
    // say what the previous window held.
    pub hint: String,
    pub href: String,
    pub tone: &'static str,
}

// Why: an absent delta is not a zero delta. `None` means the tile has nothing
// honest to compare against, and it renders no delta element at all rather
// than an em dash a reader could take for a measured no-change.
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
        sub: spec.sub,
        hint: spec.hint,
        href: spec.href,
        tone: spec.tone,
        delta: d.map(|d| d.display()).unwrap_or_default(),
        delta_dir: d.map_or("", delta_dir),
    }
}

// Why: the prior window's own figure belongs on every tile; the reason the
// percentage is missing belongs above the strip, said once. Repeating it ten
// times wrapped every tile onto a second line and cost the whole row its
// height budget, to tell the reader the same thing ten times.
pub(super) fn comparison(prior: &str, vs: &str) -> String {
    format!("{prior} {vs}")
}

// Why: the delta component takes one direction class, not a Delta — polarity
// and direction fold into that token here so the template branches on nothing.
// A flat delta gets no modifier and keeps the base arrow.
fn delta_dir(d: Delta) -> &'static str {
    match (d.direction, d.tone) {
        ("up", "bad") => "up-bad",
        ("down", "good") => "down-good",
        ("up", _) => "up",
        ("down", _) => "down",
        _ => "",
    }
}

// Why: an empty window has no rate to report; a zero would claim a
// measurement nobody took.
pub(super) fn percent_tenths(tenths: i64, sample: i64) -> String {
    if sample <= 0 {
        return "\u{2014}".to_owned();
    }
    format!("{}.{}%", tenths / 10, tenths % 10)
}
