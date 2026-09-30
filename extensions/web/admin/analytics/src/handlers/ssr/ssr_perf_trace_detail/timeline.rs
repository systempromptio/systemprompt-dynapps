//! Timeline geometry for the trace page.
//!
//! One strip, two lanes: gateway calls on top, everything else (tool events,
//! standalone decisions) as ticks below. It is an overview, not the record —
//! a 12-minute session with 47 calls shows its bursts and its failures at a
//! glance, and the turns table beneath carries the numbers. The viewBox is a
//! fixed 1000 units wide, so every x is a permille of the first-to-last
//! window and the SVG stretches to the card with no script.

use serde::Serialize;

use crate::repositories::traces::{Span, SpanKind};
use systemprompt_web_admin_core::ssr::format::format_duration_ms;

const VIEW_WIDTH: f64 = 1000.0;
const LANE_HEIGHT: f64 = 18.0;
const LANE_GAP: f64 = 6.0;
const VIEW_HEIGHT: f64 = LANE_HEIGHT * 2.0 + LANE_GAP;
const MIN_MARK_WIDTH: f64 = 3.0;
const TICK_WIDTH: f64 = 2.0;

#[derive(Debug, Serialize)]
pub(super) struct TimelineView {
    pub(super) view_width: f64,
    pub(super) view_height: f64,
    pub(super) marks: Vec<TimelineMark>,
    pub(super) ticks: Vec<TimelineTick>,
    pub(super) has_marks: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct TimelineMark {
    pub(super) id: String,
    pub(super) kind: &'static str,
    pub(super) status: &'static str,
    pub(super) label: String,
    pub(super) url: Option<String>,
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) width: f64,
    pub(super) height: f64,
}

#[derive(Debug, Serialize)]
pub(super) struct TimelineTick {
    pub(super) x: f64,
    pub(super) label: String,
}

pub(super) fn build_timeline(spans: &[Span], total_ms: i64) -> TimelineView {
    let start = spans.iter().map(|s| s.started_at).min();
    let marks: Vec<TimelineMark> = spans
        .iter()
        .filter(|s| !(matches!(s.kind, SpanKind::Governance) && s.trace_id.is_some()))
        .map(|s| {
            let offset_ms = start.map_or(0, |a| (s.started_at - a).num_milliseconds().max(0));
            let is_call = matches!(s.kind, SpanKind::Model);
            let (x, width) = if total_ms > 0 {
                let x = round2(offset_ms as f64 / total_ms as f64 * VIEW_WIDTH);
                let w = round2(s.duration_ms as f64 / total_ms as f64 * VIEW_WIDTH);
                let min = if is_call { MIN_MARK_WIDTH } else { TICK_WIDTH };
                (x.min(VIEW_WIDTH - min), w.max(min))
            } else {
                (0.0, MIN_MARK_WIDTH)
            };
            let url = match s.kind {
                SpanKind::Model => Some(format!("/admin/requests/{}", urlencoding::encode(&s.id))),
                SpanKind::Governance => Some(format!(
                    "/admin/governance/decisions/{}",
                    urlencoding::encode(&s.id)
                )),
                _ => None,
            };
            TimelineMark {
                id: s.id.clone(),
                kind: s.kind.as_str(),
                status: s.status.as_str(),
                label: format!(
                    "{} · +{} · {}",
                    s.name,
                    format_duration_ms(offset_ms),
                    format_duration_ms(s.duration_ms)
                ),
                url,
                x,
                y: if is_call { 0.0 } else { LANE_HEIGHT + LANE_GAP },
                width: width.min(VIEW_WIDTH - x),
                height: LANE_HEIGHT,
            }
        })
        .collect();

    TimelineView {
        view_width: VIEW_WIDTH,
        view_height: VIEW_HEIGHT,
        ticks: build_ticks(total_ms),
        has_marks: !marks.is_empty(),
        marks,
    }
}

fn build_ticks(total_ms: i64) -> Vec<TimelineTick> {
    (0..=4)
        .map(|i| {
            let fraction = f64::from(i) / 4.0;
            TimelineTick {
                x: round2(fraction * VIEW_WIDTH),
                label: format_duration_ms((total_ms as f64 * fraction).round() as i64),
            }
        })
        .collect()
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
